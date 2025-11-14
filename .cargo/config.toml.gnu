[build]
# Use the GNU toolchain instead of MSVC
target = "x86_64-pc-windows-gnu"

[target.x86_64-pc-windows-gnu]
linker = "gcc"
# Linker flags to avoid .drectve issues
rustflags = [
    "-C", "link-arg=-Wl,--no-insert-timestamp",
    "-C", "link-arg=-Wl,--no-gc-sections"
]
