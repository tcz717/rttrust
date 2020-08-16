# NUCLEO-F446ZE Rust Example

### How to configure your project to use Rust

#### Create rust crate

Create a rust crate under your project root folder

```bash
cargo new --lib CARGO_NAME
``` 

#### Add `SConscript`

Go to the created crate folder and create a `SConscript` file

```python
Import('RTT_ROOT')
Import('rtconfig')
from building import *

# change it if you want to use a different chip
llvm_target = 'thumbv7em-none-eabihf'

cargo = Builder(action = [
        'cargo build --manifest-path ${SOURCE.abspath} --target ${LLVM_TARGET} --target-dir ${TARGET.dir.abspath}',
        Copy('${TARGET.abspath}', '${TARGET.dir.abspath}/${LLVM_TARGET}/debug/${TARGET.file}')
    ],
    suffix = '.a',
    src_suffix = '.toml',
    prefix = 'lib',
    chdir = 1)

Env.Append(BUILDERS = {'Cargo' : cargo})
Env.AppendUnique(LLVM_TARGET = llvm_target)      

cwd = GetCurrentDir()
src = Glob('*.c')
CPPPATH = [cwd, ]

# 'rust_example' is ".a" file name
rttrust = Env.Cargo('rust_example', 'Cargo.toml')
Env.AlwaysBuild(rttrust)

group = DefineGroup('Rust', src, depend = [''], LIBS = [rttrust], CPPPATH = CPPPATH, LINKFLAGS = ' -z muldefs')

Return('group')
```

#### Add header file

C code need a header file to call rust code entry. The example uses `rust.h`:

```c
void rust_main();
```

#### Modify `Cargo.toml` file

In your `Cargo.toml` file, add `rttrust` in dependency section

```toml
[dependencies]
# path to rttrust cargo
rttrust = { path = '../../../', version = "^0" }
```

To link with the final output, the crate type has to be `staticlib`

```toml
[lib]
crate-type = ["staticlib"]
```

Cuurently, rust has no way to support unwinding in `no_std` environment. So, we need to disable panic unwinding

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

#### Modify `lib.rs` code

You need to start you code logic from `rust_main` function:

```rust
#![no_std]

// println! marco is defined in rttrust crate
#[macro_use]
extern crate rttrust;

// no mangle the name
#[no_mangle]
pub extern "C" fn rust_main() {
    println!("hello rust");
}
```