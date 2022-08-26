# discord_profile_banner_cropper

## Feature
Everyday auto crop banner image in offset loop (default is 10, build from source to change) and patch to your Discord profile (req. Discord Nitro).

## Usage
Note: The release only work on 64-bit Windows, you need build from source for other OS.  
  
1. Create a folder, download `discord_profile_banner_cropper.exe` from releases, then put it in the folder.
2. Create a folder named "src" at the same level of `discord_profile_banner_cropper.exe`.
3. Put a JPEG image named "source.jpeg" in `src` folder.
4. Create ".env" at the same level of `discord_profile_banner_cropper.exe`.
5. Put your Discord user token in `.env`, follow format `DISCORD_USER_TOKEN = "token_here"`
6. Run `discord_profile_banner_cropper.exe`.  
  
Important: DO NOT delete `cropped.jpeg` in `src`.
