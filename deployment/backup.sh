#!/bin/sh
DBNAME="radio"
NAME="/var/www/backups/backup_$(hostname -s)_$(date +"%Y-%m-%d").sql"
mysqldump ${DBNAME} --ignore-table=${DBNAME}.StationCheckHistory  --ignore-table=${DBNAME}.StationClick > ${NAME}
mysqldump --no-data ${DBNAME} StationCheckHistory StationClick >> ${NAME}
gzip -f ${NAME}
rm /var/www/backups/latest.sql.gz
ln -s "${NAME}.gz" /var/www/backups/latest.sql.gz
