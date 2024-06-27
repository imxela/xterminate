<br><br>

<p align="center"><img src="https://github.com/imxela/xterminate/blob/main/images/logo.png?raw=true" alt="xterminate logo" border="0"></p>

**<p align="center">Easily terminate any windowed process by the press of a button</p>**

<br>

<p align="center"><a href="#download">Download</a></p>

<br>

<p align="center"><img src="https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExencxYjkwZjZmaXY3NWxnb3pwbHIyN2d0NmJsM3FqMHEzdnVvYjdxcyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/ptbmSkcn2GeRzlLjId/giphy.gif" border="0"></p>

<br><br>

---

### The What

 - Terminate any unresponsive window using keyboard shortcuts
 - Always responsive to keyboard shortcuts thanks to using raw input
 - An easy-to-use tray menu for configuring settings and preferences
 - Optional Start-with-Windows functionality and automatic updates
 - Lightweight in both disk size and runtime resource consumption

### The How

 - Terminate the currently focused window: `CTRL + ALT + F4`
 - Terminate a window by clicking on it: `CTRL + ALT + END`
 - Shortcuts can be changed in the TOML configuration file

### The Why

I created xterminate to solve a frustrating issue where, upon crashing or hanging, some full-screen windows would stay on top of all other windows while blocking input. This problem often made it impossible to use the built-in task manager to close the window, since it would display below it, even with the task manager set to be always on top.

By relying on raw-input keyboard shortcuts, xterminate completely bypasses my issues with the task manager not working, making it much more reliable at closing unresponsive processes. As long as the system itself is responsive, xterminate can terminate any window with the simple press of a button.

---
## Download

You can download the latest pre-built xterminate binaries directly using the links below.

 - <a href="https://github.com/imxela/xterminate/releases/latest/download/xterminate-setup.exe">Download Installer</a>
 - <a href="https://github.com/imxela/xterminate/releases/latest/download/xterminate-portable.zip">Download Portable</a>

If you are unsure which version to get, I recommend the more user-friendly installer option. You can read release notes or download older versions of xterminate on the <a href="https://github.com/imxela/xterminate/releases">releases page</a>.

---
## Building from source

> [!IMPORTANT]  
> Some prior knowledge is assumed when building from source, such as basic command-line usage and knowledge of how to modify your PATH environment variable.

### 1. Prerequisites<a id='prerequisites'></a>

Before building xterminate, download and install the [Rust programming language](https://www.rust-lang.org/tools/install), and clone this repository to a location of your choice, either by using [Git](https://git-scm.com/downloads) or by downloading the repository as a ZIP-archive from [here](https://github.com/imxela/xterminate/archive/refs/heads/main.zip). You will also need to download and install the [Null-Soft Install System](https://nsis.sourceforge.io/Download) to compile the install script, as well as add the installation directory to your PATH.

### 2. Building

First and foremost, navigate to a location of your choice using your terminal and clone the xterminate repository.

```cmd
cd %UserProfile%\Desktop
git clone https://github.com/imxela/xterminate.git
```

Navigate into the cloned repository and run the build command to create the xterminate executable. When the build completes, the executable will be in the `xterminate-main\target\release` directory.

```cmd
cd xterminate-main
cargo build --release
```

Next, you will need to copy the `res` directory and `LICENSE` files located in the root directory of the repository to the same directory as the xterminate executable.

```cmd
robocopy .\res .\target\release\res /E
robocopy . .\target\release LICENSE
```

The `xterminate.exe` executable, the `LICENSE` file, and the `res` directory make up the portable version of xterminate. As long as they are placed alongside each other in the same directory, you can run xterminate regardless of location.

### 3. Creating the installer

> [!IMPORTANT]  
> This will only work if you added your NSIS installation directory to your PATH — see the [Building from source](#building-from-source) section and the [Prerequisites](#prerequisites) section for more information.

In your terminal, navigate to the root directory of the cloned repository and run the following command to compile the installer.

```cmd
makensis nsis\installer.nsi
```

When the compilation is complete, an installer executable is created at `target\release\xterminate-setup.exe`. Run the executable to install xterminate.

---

## QA-style information

### Q: Why does xterminate need administrator privileges?
**A:** A process can only terminate other processes if it has administrator privileges. As such, xterminate requires these privileges to function.

### Q: My antivirus software flags xterminate as malware — why?
**A:** [This is a false-positive triggered by the NSIS installer](https://nsis.sourceforge.io/NSIS_False_Positives). Unfortunately, it is not something I can fix, but rest assured, xterminate is not malicious software. The code is here for everyone to see, after all. :D

---

## License

This software and code is licensed under the terms of the MIT license. See the [LICENSE](license) file for more information.
