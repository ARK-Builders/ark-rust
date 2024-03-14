# Ark-CLI

### Installation

To compile you will need openssl libraries and headers:

```shell
# macOS (Homebrew)
$ brew install openssl@3

# macOS (MacPorts)
$ sudo port install openssl

# macOS (pkgsrc)
$ sudo pkgin install openssl

# Arch Linux
$ sudo pacman -S pkg-config openssl

# Debian and Ubuntu
$ sudo apt-get install pkg-config libssl-dev

# Fedora
$ sudo dnf install pkg-config perl-FindBin openssl-devel

# Alpine Linux
$ apk add pkgconfig openssl-dev
```

### Usage

```shell
ark-cli <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    backup        
    collisions    
    help          Print this message or the help of the given subcommand(s)
    link          
    monitor       
    render        

```

#### Backup
```shell
USAGE:
    ark-cli backup [ROOTS_CFG]

ARGS:
    <ROOTS_CFG>    

OPTIONS:
    -h, --help    Print help information
```

#### Collisions
```shell
USAGE:
    ark-cli collisions [ROOT_DIR]

ARGS:
    <ROOT_DIR>    

OPTIONS:
    -h, --help    Print help information
```

#### Link
```shell
USAGE:
    ark-cli link <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    create    
    help      Print this message or the help of the given subcommand(s)
    load
```

#### Monitor
```shell
USAGE:
    ark-cli monitor [ARGS]

ARGS:
    <ROOT_DIR>    
    <INTERVAL>    

OPTIONS:
    -h, --help    Print help information
```

#### Render
```shell
USAGE:
    ark-cli render [ARGS]

ARGS:
    <PATH>       
    <QUALITY>    

OPTIONS:
    -h, --help    Print help information

```