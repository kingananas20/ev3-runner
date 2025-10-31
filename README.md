# EV3 Runner

EV3 Runner is a Rust-based application designed to interact with EV3 robots over SSH. It provides a web server interface to upload and execute scripts on the robot, leveraging modern Rust libraries for performance and reliability.

## Features

- **SSH Integration**: Establishes secure SSH connections to EV3 robots.
- **Script Execution**: Uploads and executes scripts remotely on the robot.
- **Web Server**: Provides an HTTP endpoint for script management.
- **Streaming Logs**: Streams execution logs in real-time using Server-Sent Events (SSE).
- **Error Handling**: Implements robust error handling with `anyhow` and `thiserror`.

## Prerequisites

- Rust (edition 2024)
- EV3 robot with SSH enabled

## Installation

1. Clone the repository:
   ```sh
   git clone <repository-url>
   cd ev3-runner
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

## Usage

### Running the Application

To start the application, use:
```sh
cargo run -- --host <robot-ip> --username <robot-username> --password <robot-password> --server-port <port>
```

- Replace `<robot-ip>`, `<robot-username>`, and `<robot-password>` with your EV3 robot's details.
- The default server port is `6767`.

### HTTP API

#### `/run` (POST)
Uploads and executes a script on the robot.

**Request Body**:
```json
{
  "src_path": "/path/to/local/script",
  "dst_path": "/path/on/robot"
}
```

**Response**:
- Streams logs from the script execution in real-time.

### Testing

Run the test script:
```sh
./test.sh
```

## Dependencies

The project uses the following Rust libraries:
- `anyhow`: Error handling.
- `axum`: Web framework.
- `clap`: Command-line argument parsing.
- `futures`: Asynchronous programming.
- `serde` and `serde_json`: JSON serialization/deserialization.
- `ssh2`: SSH client library.
- `tokio`: Asynchronous runtime.
- `tracing` and `tracing-subscriber`: Logging and diagnostics.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
