#!/bin/bash
# Reset terminal after a TEXT40 display program exits improperly

# Reset terminal modes
printf '\033c'        # Reset terminal
printf '\033[?25h'    # Show cursor
printf '\033[0m'      # Reset colors
stty sane            # Reset terminal settings
clear                # Clear screen

echo "Terminal has been reset."