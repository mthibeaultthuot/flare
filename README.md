# Flare (Alpha)


> **Early Alpha** - Project under active development - API is unstable and subject to change

Flare is a custom DSL language focused on kernel fusion, memory safety, and cross-platform GPU compilation (CUDA/Metal).

## Example

```rust
let source = r#"
    kernel matmul<T>(A: Tensor<T, [M, K]>, B: Tensor<T, [K, N]>) -> Tensor<T, [M, N]> {
        grid: [M / 16, N / 16]
        block: [16, 16]

        shared_memory {
            A_tile: [16, 16]
            B_tile: [16, 16]
        }

        compute {
            var acc: T = 0
            let row = block_idx.y * 16 + thread_idx.y
            let col = block_idx.x * 16 + thread_idx.x
            output[row, col] = acc
        }
    }
"#;

let result = Flare::compile_from_string(source);
if let Err(e) = &result {
    eprintln!("Error: {:?}", e);
}
```


## Current status :
- [x] flare
- [x] flare-codegen-cuda
- [x] flare-codegen-metal
- [ ] flare-codegen-hip
- [ ] flare-codegen-tenstorrent
- [ ] flare-py-bindings
- [ ] flare-analyst
- [ ] flare-jit
- [ ] flare-std
- [ ] flare-tracing
- [ ] flare-ir
- [ ] flare-cli
