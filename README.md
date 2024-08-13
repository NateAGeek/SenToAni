# SenToAni

SenToAni is a language-learning application designed to help users learn new languages through immersive experiences with videos and subtitles. The project combines powerful tools like OpenGL for rendering, FFmpeg for video decoding, and Slint for a sleek, responsive UI. Additionally, SenToAni integrates a Language Learning Model (LLM) as a tutor to guide users through their learning journey.

## Features

- **Video-Based Learning**: Watch videos with subtitles in your target language to improve comprehension and vocabulary.
- **Interactive Subtitles**: Click on any word or phrase in the subtitles to get instant translations, definitions, and pronunciation guides.
- **LLM-Powered Tutor**: Ask questions, get explanations, and practice conversations with an integrated LLM that adapts to your learning pace.
- **Customizable UI**: Enjoy a smooth and responsive interface powered by Slint, allowing for seamless navigation and customization.
- **High-Performance Video Rendering**: Leveraging OpenGL and FFmpeg, SenToAni ensures high-quality video playback with minimal latency.

## Goals Checklist

- [x] **Get a video playing in Slint using OpenGL**
  - Integrate OpenGL with Slint to render videos smoothly.
  - Decode videos using FFmpeg and display them within the Slint UI.
  - TODO: Better support multiple formats, maybe break off into new crate for video playback?

- [x] **Display subtitles using Slint**
  - Parse subtitle files (e.g., SRT, ASS) and render them in sync with the video.
  - TOOD: Ensure subtitles are customizable in terms of font, size, and color.

- [ ] **Incorporate an LLM to interact with the video**
  - Implement a feature where the LLM can pause the video at key moments to ask questions or provide explanations.
  - Allow the user to ask questions to the LLM about the content being watched, including vocabulary, grammar, and cultural context.

## Getting Started

### Prerequisites

- **Rust**: Make sure you have Rust installed on your machine. You can install it using [rustup](https://rustup.rs/).
- **FFmpeg**: Install FFmpeg for video decoding. Refer to the [FFmpeg installation guide](https://ffmpeg.org/download.html) for your platform.
- **OpenGL**: Ensure you have the necessary OpenGL drivers installed.

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/NateAGeek/SenToAni.git
   cd sentoani
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Run the application:

   ```bash
   cargo run
   ```

### Usage

- **Load a Video**: Load a video file and select the subtitle track you want to follow along with.
- **Interact with Subtitles**: Click on any word in the subtitle track to get additional information and learning resources.
- **Ask the Tutor**: Use the built-in LLM to ask questions about grammar, vocabulary, or even cultural context.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is still to be licensed. But right now we can go under "WTFPL" but soon to change.