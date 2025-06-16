# rdicom-lite
Minimal, fast, and extensible Rust DICOM toolkit for automating and optimizing DICOM workflows

# Usage
## CLI
```bash
rdicom-lite [PATH] [COMMAND] [MODE] [EXTRA]
```
## ⚙ Arguments

| Argument   | Description                                                                                  |
|------------|----------------------------------------------------------------------------------------------|
| `PATH`     | Path to a single DICOM file or a directory. If a directory is provided, all `.dcm` files inside will be processed. |
| `COMMAND`     | specific command to process the files.|
| `MODE`     | Defines how metadata should be displayed. **See specific command behaviour for mode and extra.**|
| `EXTRA`    | Optional, command & mode-specific arguments (e.g., exporting).                             |

### 🔖 Tag Mode
Use the `tags` command to display or export metadata.

```bash
rdicom-lite <PATH> tags <MODE> [EXTRA]
```

### Available `MODE` options:

| Mode | Description |
|---|---|
|`all`| Displays **all** available DICOM tags in a detailed format.                                   |
|`short`| Displays a predefined short list of common tags: `PatientName`, `StudyDate`, `Modality`, `SeriesDescription`.|
|`specific`| Act as the `EXTRA`. Displays only selected tags, comma-separated. Tag names are case-insensitive.|

#### Example using `specific` mode:
```bash
rdicom-lite ./scan.dcm tags PatientName,PatientID,StudyDate
```

### Available `EXTRA` options
Can add either one or all to export the usual output as serialized JSON or CSV.
If encounter a file at this path, create a new one
| Mode | Description |
|---|---|
|`--json=[PATH]`| Export as JSON  |
|`--csv=[PATH]`|  Export as CSV|

The outputed data consists of:
- filename
- name
- tag group / element
- vr
- value



# Goals
## Base of CLI (Goals for v0.1)
- [x] tags:
    - [x] all  : Dumps all tags
    - [x] "tag": for a single tag
    - [x] short:  a curated list of “high-value” tags (PatientID, etc...)
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
