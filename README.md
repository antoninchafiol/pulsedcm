# Project Overview
**pulsedcm** is a high-performance, memory-safe Rust toolkit for DICOM automation and AI-ready workflows.  
Designed for researchers, imaging scientists, and developers working with large volumes of medical imaging data.

It focuses on being:

- ⚡ **Blazingly Fast** – optimized for parallel I/O, tag parsing, anonymization, and PNG export
- 🦀 **Safe** – uses Rust’s guarantees to avoid crashes and undefined behavior in production
- 🧱 **Modular & Scalable** – structured as a set of composable crates for integration or expansion


I plan to build this to be accelerating workflows whether it be machine learning or research toolchain and avoid bottlenecking on DICOM parts/exchanges.


<details> 
<summary> <h2> Usage </h2> </summary>


```bash
pulsedcm [PATH] [COMMAND] [MODE] [EXTRA]
```
### ⚙ Arguments

| Argument   | Description                                                                                  |
|------------|----------------------------------------------------------------------------------------------|
| `PATH`     | Path to a single DICOM file or a directory. If a directory is provided, all `.dcm` files inside will be processed. |
| `COMMAND`     | specific command to process the files.|
| `MODE`     | Defines how metadata should be displayed. **See specific command behaviour for mode and extra.**|
| `EXTRA`    | Optional, command & mode-specific arguments (e.g., exporting).                             |

<details>
<summary> <h3> 🔖 Tag Mode </h3> </summary>
Use the `tags` command to display or export metadata.

```bash
pulsedcm <PATH> tags <MODE> [EXTRA]
```

#### Available `MODE` options:

| Mode | Description |
|---|---|
|`all`| Displays **all** available DICOM tags in a detailed format.                                   |
|`short`| Displays a predefined short list of common tags: `PatientName`, `StudyDate`, `Modality`, `SeriesDescription`.|
|`specific`| Act as the `EXTRA`. Displays only selected tags, comma-separated. Tag names are case-insensitive.|

##### Example using `specific` mode:
```bash
pulsedcm ./scan.dcm tags PatientName,PatientID,StudyDate
```

#### Available `EXTRA` options
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

</details>
<details>

<summary> <h3> 🖼 View Mode </h3> </summary>

**Warning:** As using the dicom-rs crate, the view mode is restricted and unable to decode JPEG 2000 Lossless compression as of now.

Use the `view` command to render DICOM slices as PNGs and open them with your OS’s default image viewer.

```bash
pulsedcm <PATH> view [OPTIONS]

```
#### Options

| Option            | Description                                                                                            |
| ----------------- | ------------------------------------------------------------------------------------------------------ |
| `--open <NUMBER>` | Number of images to open via the OS’s PNG viewer (e.g. `--open 5` opens the first five PNGs).          |
| `--temp`          | Write PNGs to the system temporary directory instead of alongside the DICOM files. Implies `--open 1`. |
| `--jobs <NUMBER>` | Number of threads to launch for parallel processing of slices.                                         |
| `-h`, `--help`    | Print this help message.                                                                               |

</details>
<details>
<summary> <h3> 🔒 <b>Ano</b>nymization / De-identification   </h3> </summary>


<div style="background-color: #cce5ff; border: 1px solid #b8daff; padding: 12px; border-radius: 4px; margin: 16px 0;">
ℹ️ <b>Note</b>: This is using the DICOM Supplement 142 Standard de-identification methods.
</div>
<div style="background-color: #fff3cd; border: 1px solid #ffeeba; padding: 12px; border-radius: 4px; margin: 16px 0;">
⚠️ <b>Warning</b>: No pixel modification for this version.
</div>

<h5>Usage</h5>

```bash
pulsedcm <PATH> ano [OPTIONS]
```

#### Options

| Option              | Description                                                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `--action <ACTION>` | See **Action Types** table below.                                                                                              |
| `--policy <POLICY>` | See **Policy Types** table below.                                                                                              |
| `--jobs <NUMBER>`   | Number of threads to launch to process (0 or less = all available threads)                                                             |
| `--out <OUT>`       | Output directory to save anonymized files. If omitted, input files are overwritten in-place. Must be a directory if specified. |
| `-d`, `--dry`       | Show the changed args for the file. If multiple files, stops after the first to display output.                                |
| `-v`, `--verbose`   | Show all changed values.                                                                                                       |
| `-h`, `--help`      | Print this help message.                                                                                                       |

#### Action Types

| Action    | Description                                        |
| --------- | -------------------------------------------------- |
| `replace` | Replace the tag’s value with a dummy value.        |
| `zero`    | Replace the tag’s value with a zero-length string. |
| `remove`  | Remove the tag entirely.                           |

#### Policy Types

| Policy     | Description                                            |
| ---------- | ------------------------------------------------------ |
| `basic`    | Remove only the required PHI elements (safe profile).  |
| `moderate` | Also remove institution and device information.        |
| `strict`   | Maximum removal: leaves only technical/essential data. |



</details>
</details>

<details>
<summary> <h2> 🎯 Roadmap & Progress </h2> </summary>

## Base of CLI (Goals for v0.1)
- [x] tags:
    - [x] all  : Dumps all tags
    - [x] "tag": for a single tag
    - [x] short:  a curated list of “high-value” tags (PatientID, etc...)
- [x] view: Display the image (ASCII/Invoke OS' Viewer)
- [x] anonymize: Remove PHI


## Intermediate (Goals for v0.2)
- [ ]: Feature-Gated Modules: creating crates modules to add for a personalized executable.
- [ ]: Optimize: tags
- [ ]: Optimize: view
- [ ]: Optimize: ano

## Advanced (Goals for v0.3)
#### CLI
- [ ] PACS Interop:
    - [ ] send: Send to a server
    - [ ] recv: Declare a server 

## Extensions & Polish
- [ ] Implement DICOMWeb
</details>



