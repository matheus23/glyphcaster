use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use automerge::{
    ChangeHash, PatchLog, ROOT, ReadDoc, TextEncoding, patches::TextRepresentation,
    transaction::Transactable,
};
use futures::StreamExt;
use glib::spawn_future_local;
use gtk::prelude::{TextBufferExt, TextBufferExtManual};
use sourceview5::Buffer;

#[derive(Clone)]
pub(crate) struct TextSynchronizer {
    handle: samod::DocHandle,
    router: iroh::protocol::Router,
    editor_buffer: Buffer,
    reconciling: Arc<AtomicBool>,
    view_heads: Arc<Mutex<Vec<ChangeHash>>>,
}

impl TextSynchronizer {
    pub(crate) fn new(
        handle: samod::DocHandle,
        buffer: Buffer,
        router: iroh::protocol::Router,
    ) -> Self {
        let view_heads = handle.with_document(|doc| doc.get_heads());
        Self {
            handle,
            router,
            editor_buffer: buffer,
            reconciling: Arc::new(AtomicBool::new(false)),
            view_heads: Arc::new(Mutex::new(view_heads)),
        }
    }

    fn handle_splice(&self, insert: usize, delete: usize, text: &str) {
        if self.reconciling.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        let mut view_heads = self.view_heads.lock().unwrap();
        self.handle.with_document(|doc| {
            let mut tx = doc.transaction_at(
                PatchLog::inactive(TextRepresentation::String(TextEncoding::GraphemeCluster)),
                &view_heads.as_ref(),
            );
            let (_, text_obj_id) = tx
                .get(ROOT, "content")
                .expect("failed to get content object")
                .expect("no content key found");
            tx.splice_text(text_obj_id, insert, delete as isize, text)
                .unwrap();
            let (new_head, _) = tx.commit();
            if let Some(new_head) = new_head {
                *view_heads = vec![new_head];
            }
        });
        drop(view_heads);
        self.reconcile();
    }

    fn reconcile(&self) {
        self.reconciling.store(true, Ordering::Release);

        let mut view_heads = self.view_heads.lock().unwrap();

        let text_obj_id = self.handle.with_document(|doc| {
            doc.get(ROOT, "content")
                .expect("failed to get content object")
                .expect("no content key found")
                .1
        });

        let (diff, new_heads) = self.handle.with_document(|doc| {
            let heads = doc.get_heads();
            let patches = doc.diff(
                view_heads.as_ref(),
                &heads,
                TextRepresentation::String(TextEncoding::GraphemeCluster),
            );
            (patches, heads)
        });
        let mut index_adjustment = 0;
        for patch in diff {
            if patch.obj != text_obj_id {
                continue;
            }
            tracing::debug!(?patch, "applying patch");
            match patch.action {
                automerge::PatchAction::SpliceText {
                    index,
                    value,
                    marks: _,
                } => {
                    let index = index + index_adjustment;
                    let mut pos = self.editor_buffer.iter_at_offset(index as i32);
                    let as_text = value.make_string();
                    self.editor_buffer.insert(&mut pos, &as_text);
                    index_adjustment += as_text.len();
                }
                automerge::PatchAction::DeleteSeq { index, length } => {
                    let index = index + index_adjustment;
                    let mut start = self.editor_buffer.iter_at_offset(index as i32);
                    let mut end = self.editor_buffer.iter_at_offset((index + length) as i32);
                    self.editor_buffer.delete(&mut start, &mut end);
                }
                _ => {}
            }
        }
        *view_heads = new_heads;
        self.reconciling.store(false, Ordering::Release);
    }

    pub(crate) fn start(&self) {
        // Wire up insertion
        {
            let this = self.clone();
            self.editor_buffer
                .connect_insert_text(move |_buffer, location, text| {
                    this.handle_splice(location.offset() as usize, 0, &text);
                });
        }

        // And deletion
        {
            let this = self.clone();
            self.editor_buffer
                .connect_delete_range(move |_buffer, start, end| {
                    let start = start.offset() as usize;
                    let end = end.offset() as usize;
                    let len = end - start;
                    this.handle_splice(start, len, "");
                });
        }

        // Now, whenever the document changes, update the text buffer
        {
            let this = self.clone();
            spawn_future_local(async move {
                let mut changes = this.handle.changes();
                while let Some(_) = changes.next().await {
                    this.reconcile();
                }
            });
        }
    }
}
