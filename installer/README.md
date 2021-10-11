# Installer

What the installer does:

## On Windows:

1. Installs the program to `%APPDATA%\Solar Screen Brightness\`
2. Creates a start menu shortcut in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\`
3. Creates a startup shortcut in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`

## On Linux:

1. Writes the app binary to `~/.config/Solar Screen Brightness/`
2. Creates a desktop entry in `~/.local/share/applications/`
3. Creates an autostart entry in `~/.config/autostart/`


## Development

To create a release use:

```
git tag -a <TAG> <COMMIT> -m "Message"
git push origin <TAG>
```
