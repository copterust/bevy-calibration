Running:

```bash
cargo run PORT MODE
```

Where:
* PORT -- your serial port where [test firmware](https://github.com/copterust/proving-ground/tree/master/ahrs-ekf) is connected
* MODE -- one of "raw" (default -- read raw samples and calibrate) or "cal" (samples scaled at the device)