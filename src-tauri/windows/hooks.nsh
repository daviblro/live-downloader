!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Live Downloader installed for the current Windows user."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Preparing to remove Live Downloader. User recordings are preserved."
!macroend
