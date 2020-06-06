cargo build --release
SET "source=.\target\release\solar-screen-brightness.exe"
SET "destination=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\Solar Screen Brightness.exe"
COPY "%source%" "%destination%"
START "" "%destination%"
