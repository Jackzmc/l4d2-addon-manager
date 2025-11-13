pnpm tauri build && \
rsync -v src-tauri/target/release/bundle/**/*.* root@lambda:/var/www/dl.jackz.me/l4d2-addon-manager/linux/x86_64/