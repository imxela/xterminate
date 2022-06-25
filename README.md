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
  With xterminate, closing unresponsive applications becomes as easy as the press of a button. 
  All you have to do is press <code>CTRL+ALT+END</code> and xterminate will spring to action. 
  Any window you subsequently click terminates instantly. Regardless of what type of window it is, xterminate will make sure it terminates. 
  No more rebooting when apps and games go haywire.
  Once installed, xterminate will always be on standby in the background, ready for the next time you need to terminate a misbehaving application.
</p>

**<p>Main Features and Todos<p>**

  - [x] `CTRL+ALT+END`: Activate xterminate
  - [x] `Left Mouse Button`: Terminate the window under your mouse cursor (after pressing `CTRL+ALT+END`)
  - [x] `ESCAPE`: Abort termination (after pressing `CTRL+ALT+END`)
  - [x] Global, unintrusive, and uninterruptible input handling using Windows' raw input API
  - [x] A neat tray icon
  - [ ] `CTRL+ALT+F4`: Immediately terminate the currently in-focus window
  - [ ] Built-in start with Windows functionality
  - [ ] Attempt to gracefully exit before terminating
  - [ ] Configurable keys and settings

---

### Why
Sometimes Task Manager just isn't up to the task.

<p align="justify">
  Some applications can be very stubborn when they crash, hang, or freeze. Sometimes so much so that even Task Manager cannot manage.
  Even if you set Task Manager to be always on top, 
  it sometimes still displays below unresponsive applications, making it difficult to terminate 
  the application without having to resort to a system reboot. With xterminate installed, these issues become a problem of the past.
</p>

---

### Building from source
Before attempting to build xterminate, you need to [download and install Rust](https://www.rust-lang.org/tools/install).

Once Rust is installed, paste this one-liner in `cmd.exe` and it will clone xterminate to your desktop and build the code for you:

    git clone https://github.com/alexkarlin/xterminate.git "%UserProfile%\Desktop\xterminate-main" && cd "%UserProfile%\Desktop\xterminate-main" && build.bat

Alternatively, you can clone the repo yourself to wherever you want and run the `build.bat` script manually.

<p align="justify">
  Your executable will be located at <code>xterminate-main\target\release\xterminate.exe</code> along with the resource directory (<code>\res</code>). 
  Note that the installer currently handles "start with Windows" functionality, 
  so when building from source, xterminate will not start with Windows as it would otherwise.
</p>

---

### Useful QA-style information
**Q: Why does xterminate need to run as an administrator?**
<br>**A:** In order for xterminate to be able to terminate another process, it needs to share the same or higher privileges. 
As a result, running xterminate without elevated privileges will cause it to be unable to terminate some applications.

**Q: My anti-virus flags xterminate as malware, why?**
<br>**A:** [This is a false-positive triggered by the NSIS installer](https://nsis.sourceforge.io/NSIS_False_Positives).

**Q: My cursor is stuck as a red cross!**
<br>**A:** This might happen if xterminate closes unexpectedly after pressing `CTRL+ALT+END`.
Simply open your tray-icon menu, right-click xterminate's icon, and press _"Reset cursor"_ to revert to your normal cursor.

**Q: Will xterminate still work if the mouse cursor is hidden?**
<br>**A:** Yup, it will! Just make sure your mouse cursor is _somewhere_ inside the window you want to terminate before clicking.
