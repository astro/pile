## Wiring

As defined in `src/main.rs`:

+--------------|------------------------|---------------|
| Peripheral   | Data output (MOSI)     | Clock (SCK)   |
+--------------|------------------------|---------------|
| SPI1         | PB5                    | PA5           |
| SPI2         | PB15                   | PD3           |
| SPI3         | PC12                   | PC10          |
+--------------|------------------------|---------------|

Feel free to change those. The Rust compiler will catch hardware
misconfigurations through the SPI/DMA traits in `stm32f429-hal`.
