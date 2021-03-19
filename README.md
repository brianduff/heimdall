# Heimdall

During lockdown, my kids have been finding ways to be super creative about getting extra screen time, despite already having lots of zoom meetings.

I looked for existing software that would let me exercise an iron grip over the time my kids spend on their computer, but had trouble finding something that exactly fit my requirements. So I created Heimdall to meet that need.

## Concepts

Heimdall's main feature is that it can lock out a specified user account on a schedule. The account is locked out by default, and then unlocked at specific times based on a schedule set by the parent.

The lock out mechanism is implemented using password rotation. The parent sets a lockdown password which is known only to them, and while the computer is locked, the password is set to this. The parent can override the schedule at any time to either lock or unlock the computer.

## Technology

The program's core logic is implemented in Rust, and is intended to run as a daemon. It provides an http endpoint (implemented with the Rocket library) that exposes a simple API and serves the frontend configuration management UI. The frontend is implemented with html and javascript using the Vue library.

## Building / running

For development, after installing cargo, just `cargo run`; the local http URL will be printed on the console.
