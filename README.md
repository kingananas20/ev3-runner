# ev3-runner

A command-line tool to upload and run binaries on LEGO EV3 robots running ev3dev over TCP.

## Features

- 🚀 Fast binary uploads with hash-based deduplication
- 🔒 Password-protected connections
- 📦 Automatic file permission handling
- 🔄 Upload-only or upload-and-run modes
- 📡 Real-time output streaming from executed programs

## Installation

### From crates.io

```bash
cargo install ev3-runner
```

### From source

```bash
git clone https://github.com/kingananas20/ev3-runner
cd ev3-runner
cargo install --path .
```

### Pre-built EV3 Binary

Pre-compiled binaries for the EV3 (ARMv5TE architecture) are available in the [latest GitHub release](https://github.com/kingananas20/ev3-runner/releases/latest).

Download the `ev3-runner` binary from the release page and transfer it to your EV3 robot.

## Usage

### Server Mode (on EV3)

Run the server on your EV3 robot:

```bash
ev3-runner server
```

With custom port and password:

```bash
ev3-runner server --server-port 8080 --password mysecret
```

### Client Mode (on your computer)

Upload a file:

```bash
ev3-runner client upload ./my-program
```

Upload and run a file:

```bash
ev3-runner client run ./my-program
```

With custom options:

```bash
ev3-runner client run ./my-program \
  --remote-path /home/robot/my-program \
  --host 192.168.1.100:6767 \
  --password mysecret
```

### Options

#### Server Options

- `-p, --server-port <PORT>` - Port to listen on (default: 6767)
- `-p, --password <PASSWORD>` - Server password (default: maker)
- `-v` - Increase verbosity (can be repeated: `-v`, `-vv`, `-vvv`)

#### Client Options

- `-r, --remote-path <PATH>` - Target path on the server (default: same as local filename)
- `--host <HOST>` - Server address in `addr:port` format (default: 127.0.0.1:6767)
- `-p, --password <PASSWORD>` - Connection password (default: maker)
- `-v` - Increase verbosity (can be repeated: `-v`, `-vv`, `-vvv`)

## How It Works

1. **Client** calculates a hash of the local file
2. **Client** sends file metadata (path, size, hash) and password to **Server**
3. **Server** verifies the password
4. **Server** checks if the file already exists with the same hash
5. If hashes don't match, **Client** uploads the file
6. If in "run" mode, **Server** executes the binary and streams output back to **Client**

This hash-based approach avoids unnecessary uploads when the file hasn't changed, making iterative development faster.

## Example Workflow

1. Start the server on your EV3:
   ```bash
   ./ev3-runner server --password mypassword
   ```

2. From your development machine, upload and run your program:
   ```bash
   ev3-runner client run ./target/armv5te-unknown-linux-musleabi/release/my-robot-program \
     --host 192.168.1.100:6767 \
     --password mypassword
   ```

3. Watch the output stream in real-time from your EV3!

## Building for EV3

To cross-compile your Rust programs for the EV3:

```bash
rustup target add armv5te-unknown-linux-musleabi
cargo build --release --target armv5te-unknown-linux-musleabi
```

## Security Note

The password is hashed using SHA-256 before transmission. However, this tool is designed for development workflows and should not be used in security-critical environments. Always use it on trusted networks.

## License

MIT

## Repository

[https://github.com/kingananas20/ev3-runner](https://github.com/kingananas20/ev3-runner)
