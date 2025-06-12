# rdicom-lite
Minimal, fast, and extensible Rust DICOM toolkit for automating and optimizing DICOM workflows

## Base of CLI (Goals for v0.1)
- [ ] tags:
    - [ ] all  : Dumps all tags
    - [ ] "tag": for a single tag
    - [ ] short:  a curated list of “high-value” tags (PatientID, etc...)
- [ ] view: Display the image (ASCII/Invoke OS' Viewer)
- [ ] anonymize: Remove PHI


## Intermediate (Goals for v0.2)
#### CLI
- [ ] compress / decompress
- [ ] edit: Allow to edit metadata -> Open a text editor (Vim/VScode/Notepad)
#### Concept
- [ ]: Feature-Gated Modules: creating crates modules to add for a personalized executable.

## Advanced (Goals for v0.3)
#### CLI
- [ ] validation: compare two files and report changes (tags and pixels)
- [ ] PACS Interop:
    - [ ] send: Send to a server
    - [ ] recv: Declare a server 

## Extensions & Polish
- [ ] Implement DICOMWeb