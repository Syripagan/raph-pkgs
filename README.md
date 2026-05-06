# rad

rad is a source-based package manager for Radrix GNU/Linux or other GNU/Linux distros, usually LFS.

rad is an abbreviation for Rathrix Automated TOML-packages Header

but when it combines with Slavic God Radogost, who is the God of trade and seafaring,
even easier to call it just rad.

It stays for managing system packages, user ones is better to manage with nix or other

## Installation

To install it, firstly clone the repository

```sh
git clone https://github.com/dejuri/rad.git Rad
```
Then change directory to just cloned project

```sh
cd Rad
```

Build it with cargo (you might want to firstly execute cargo update)

```sh
cargo build --release
```

Then install rad into the system (execute as root)

```sh
cp ./target/release/rad /usr/bin
```

Now you have rad installed!

P.S. If you want you can install rad from rad itself now

```sh
rad -i rad
```

