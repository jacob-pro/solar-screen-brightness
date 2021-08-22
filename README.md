# Solar Screen Brightness

Varies your screen brightness according to sunrise/sunset times.

#### What is this for?
Desktop PCs and computers without an ambient light sensor.

#### How is this different to [f.Lux](https://justgetflux.com/) or similar Night Mode programs?
This changes the monitor screen brightness via the DDC/CI monitor APIs, whereas those utilities vary the colour tone of the display.

#### Platform Support
Currently windows only, however the [algorithm](https://github.com/jacob-pro/sunrise-sunset-calculator) is in portable C, the user interface uses [crossterm](https://github.com/crossterm-rs/crossterm), to make fully cross platform would require replacing some Win32 API calls with equivalents. 

#### Features
- An icon appears in the tray when running
- Clicking the tray icon opens a console UI to configure the application
- Only one instance may be started per user
- Configuration file persisted to AppData
- Is automatically disabled/enabled on user lock/unlock events

#### To do
- [ ] Per monitor/device brightness settings
- [ ] Linux Support
- [ ] Mac Support

## Screenshots

![](./screenshots/main.png)

![](./screenshots/status.png)

![](./screenshots/edit_config.png)


## Development

### RHEL

`clang ncurses-devel qt5-qtbase-devel`
`ln -s /usr/bin/qmake-qt5 /usr/bin/qmake`
