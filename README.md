# Solar Screen Brightness

Varies your screen brightness according to sunrise/sunset times.

#### What is this for?
Desktop PCs and computers without an ambient light sensor.

#### How is this different to `f.lux` or Night Light?
This changes the monitor screen brightness via the DDC/CI monitor APIs, whereas those utilities vary the colour tone of the display.

#### Platform Support
Currently windows only, however the algorithm is in portable C, the user interface uses `crossterm`, to make fully cross platform just requires replacing some Win32 API calls with equivalents. 
