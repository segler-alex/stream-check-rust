#!/bin/bash
cargo build --release
sudo mkdir -p /usr/local/bin

sudo cp target/release/stream-check-rust /usr/local/bin/stream-check.bin
sudo cp init/stream-check.service /etc/systemd/system

cat <<EOF | sudo tee /usr/local/bin/stream-check
#!/bin/bash
. /etc/stream-check.conf
export DELETE
export TCP_TIMEOUT
export PAUSE_SECONDS
export LOOP
export STATIONS
export CONCURRENCY
export SOURCE
export DATABASE_URL
stream-check.bin
EOF

cat <<EOF | sudo tee /etc/stream-check.conf
# do delete not working stations
DELETE=true
# tcp timeout for connect and read
TCP_TIMEOUT=20
# pause between database checks if no stations to check are available
PAUSE_SECONDS=60
# continue after STATIONS stations have been checked
LOOP=true
# how many stations max should be checked in each loop
STATIONS=100
# how many stations should be checked in parallel
CONCURRENCY=5
# set the source host
SOURCE=$(hostname)
# database connection string (mysql, mariadb)
DATABASE_URL=mysql://myuser:mypassword@localhost/radio
EOF

sudo chmod ugo+x /usr/local/bin/stream-check
sudo groupadd --system streamcheck
sudo useradd --system --no-create-home --gid streamcheck streamcheck

sudo systemctl daemon-reload

echo "Enable service with:"
echo " - systemctl enable stream-check"
echo "Start service with:"
echo " - systemctl start stream-check"
echo "Logs:"
echo " - journalctl log -uf stream-check"
echo "Edit /etc/stream-check.conf according to your needs."