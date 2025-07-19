# Document ID Display Feature

## Overview

This feature adds a UI element to display the document ID in the Spanreed automerge document editor. Previously, when creating a new document, users had no way to see the generated document ID.

## Changes Made

### 1. Updated `AppState` Structure (`src/app_state.rs`)

- Added new fields to store the document handle and UI components:
  - `doc_handle: Rc<RefCell<Option<DocHandle>>>` - Stores the document handle
  - `header_bar: gtk::HeaderBar` - Header bar to contain document info
  - `doc_id_label: gtk::Label` - Label to display the document ID
  - `copy_button: gtk::Button` - Button to copy the document ID to clipboard

- Modified the editor page layout:
  - Changed from horizontal to vertical orientation to accommodate the header bar
  - Added a header bar at the top of the editor page containing the document ID display

- Added new method `update_document_id()`:
  - Updates the label text with the document ID
  - Enables the copy button
  - Sets up clipboard functionality to copy the document ID when the button is clicked

### 2. Updated `DocumentLoader` (`src/document_loader.rs`)

- Modified the document loading flow to:
  - Get the document ID from the handle using `doc_handle.document_id()`
  - Store the document ID in the app state
  - Call `update_document_id()` to update the UI with the document ID

### 3. UI Design Decisions

- **Location**: The document ID is displayed in a header bar at the top of the editor
- **Format**: Shows as "Document ID: <id>" with ellipsis for long IDs
- **Interactivity**: 
  - The label is selectable for manual copying
  - A copy button provides one-click copying to clipboard
  - Tooltip shows the full document ID on hover
- **Visual Design**: Uses GTK's subtitle CSS class for appropriate styling

## User Experience

1. When opening an existing document: The document ID is displayed immediately after loading
2. When creating a new document: The generated document ID is displayed after creation
3. Users can easily copy the document ID by:
   - Clicking the copy button
   - Selecting the text manually
4. The full document ID is available via tooltip if truncated in the display

## Technical Notes

- The document ID is obtained using the `document_id()` method on `DocHandle`
- The copy functionality uses GTK's clipboard API
- The UI updates are thread-safe using RefCell for interior mutability