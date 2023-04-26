<br><br>

<p align="center"><img src="images/logo.png?raw=true" alt="xterminate logo" border="0"></p>

**<p align="center">Easily terminate any windowed process by the press of a button</p>**

<br>

<p align="center"><a href="https://github.com/alexkarlin/xterminate/releases/latest/download/xterminate-setup.exe">Download</a></p>

<p align="center">
  <img src="https://img.shields.io/github/downloads/alexkarlin/xterminate/total">
  <img src="https://img.shields.io/github/license/alexkarlin/xterminate">
</p>

<br><br>

---

### Usage

<p align="justify">
  With xterminate, all you have to do to terminate unresponsive applications is press <code>CTRL+ALT+END</code> and xterminate will enter termination mode. Any window you subsequently left-click terminates instantly. No more rebooting when apps or games go haywire. Once installed, xterminate will always be on standby in the background, ready for the next time you need to terminate a misbehaving application.
</p>

<br>

**<p>Features</p>**
  - [x] Visual feedback when entering termination mode in the form of a custom cursor
  - [x] Graceful exit and forced termination of any windowed process
  - [x] Global, uninterruptible input to ensure xterminate always remains responsive
  - [x] Configurable keys and settings in a `config.toml` file
  - [x] Optional start-with-Windows functionality
  - [x] A neat tray-icon
  - [x] Improved `ALT+F4`-equivalent with an added ability to terminate unresponsive windows

<br>

**<p>Default Keybinds</p>**
  - `CTRL+ALT+END` to enter termination mode
  - In termination mode, click `Left Mouse Button` to terminate any window
  - In termination mode, press `ESCAPE` to leave termination mode
  - `CTRL+ALT+F4` to terminate the current window in focus

All key-binds can be changed in the `config.toml` file.

---

### Why

<p align="justify">
  Some applications are very stubborn when they crash, hang, or freeze. So much so that task manager might not work. Even when set to 
  "always on top", Task Manager sometimes still displays below unresponsive applications, making it impossible to navigate the UI and to terminate a faulty application. Since xterminate does not rely on any user interface, this becomes a non-issue.
</p>

---

### Building from source
If you do not want to use the <a href="https://github.com/alexkarlin/xterminate/releases/">pre-built binaries or installer</a>, you can build xterminate from source using the instructions here.

Before attempting to build xterminate, you need to [download and install Rust](https://www.rust-lang.org/tools/install).

Once Rust is installed, paste this one-liner in `cmd.exe` and it will clone xterminate to your desktop and build the code for you in one go:

    git clone https://github.com/alexkarlin/xterminate.git "%UserProfile%\Desktop\xterminate-main" && cd "%UserProfile%\Desktop\xterminate-main" && build.bat

Alternatively, you can clone the repo yourself to wherever you want and run the `build.bat` script manually.

<p align="justify">
  Your executable will be located at <code>xterminate-main\target\release\xterminate.exe</code> along with the resource directory (<code>\res</code>). 
</p>

Note: Make sure to always place the executable (`xterminate.exe`) and the resource directory (`/res`) in the same root directory.

---

### Useful QA-style information
**Q: Why does xterminate need to run as an administrator?**
<br>**A:** In order for xterminate to be able to terminate another process, it needs to share the same or higher privileges. As a result, running xterminate without elevated privileges will cause it to be unable to terminate some applications.

**Q: My anti-virus flags xterminate as malware, why?**
<br>**A:** [This is a false-positive triggered by the NSIS installer](https://nsis.sourceforge.io/NSIS_False_Positives).

**Q: My cursor is stuck as a red cross!**
<br>**A:** This might happen if xterminate closes unexpectedly after pressing `CTRL+ALT+END`.
Simply open your tray-icon menu, right-click xterminate's icon, and press _Reset cursor_ to revert back to your normal cursor.

**Q: Will xterminate still work if the mouse cursor is hidden?**
<br>**A:** Yup, it will! Just make sure your mouse cursor is _somewhere_ inside the window you want to terminate before clicking (or use `CTRL+ALT+F4` instead!).
