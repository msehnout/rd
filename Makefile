sync:
	watchexec -e rs,toml -i target/ --force-poll 1000 'rsync -rvu --exclude target /var/macos/rd /home/vagrant/'
