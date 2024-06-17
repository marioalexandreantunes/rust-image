# Template Matching in Rust

This repository contains a Rust implementation of a template matching algorithm to locate small images within a larger image. Template matching is a technique in digital image processing for finding small parts of an image that match a template image.
The goal is to be as fast as possible using parallelism and several small images within a larger image, perfect to use in bots.

## Features

- **Load the source image and the template images:** parsing some infos from template names, like tolerance level and percentage
- **Template Matching Algorithm:** Efficiently searches and identifies regions in a larger image that match a given smaller template image.
- **Image Processing with Rust:** Leverages Rust's performance and safety to handle image processing tasks.
- **Example Usage:** Includes examples to demonstrate how to use the library for template matching.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - Ensure you have the Rust toolchain installed.

### Installation

Add this repository as a dependency in your `Cargo.toml`:

```toml
[dependencies]
image = "0.25.1"
imageproc = "0.25"
rayon = "1.5"

