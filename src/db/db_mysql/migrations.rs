use crate::db::db_mysql::simple_migrate::Migrations;

pub fn load_migrations(pool: &mysql::Pool) -> Result<Migrations, Box<dyn std::error::Error>> {
    let mut migrations = Migrations::new(pool);
    migrations.add_migration("20190104_014300_CreateStation",
r#"CREATE TABLE `Station` (
`StationID` int(11) NOT NULL AUTO_INCREMENT,
`Name` text COLLATE utf8mb4_unicode_ci,
`Url` text,
`Homepage` text,
`Favicon` text,
`Creation` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
`Country` varchar(50) DEFAULT NULL COLLATE utf8mb4_unicode_ci,
`Language` varchar(50) DEFAULT NULL COLLATE utf8mb4_unicode_ci,
`Tags` text COLLATE utf8mb4_unicode_ci,
`Votes` int(11) DEFAULT '0',
`NegativeVotes` int(11) NOT NULL DEFAULT '0',
`Subcountry` varchar(50) DEFAULT NULL COLLATE utf8mb4_unicode_ci,
`clickcount` int(11) DEFAULT '0',
`ClickTrend` int(11) DEFAULT '0',
`ClickTimestamp` datetime DEFAULT NULL,
`Codec` varchar(20) DEFAULT NULL,
`LastCheckOK` tinyint(1) NOT NULL DEFAULT '1',
`LastCheckTime` datetime DEFAULT NULL,
`Bitrate` int(11) NOT NULL DEFAULT '0',
`UrlCache` text NOT NULL,
`LastCheckOkTime` datetime DEFAULT NULL,
`Hls` tinyint(1) NOT NULL DEFAULT '0',
`IP` varchar(50) NOT NULL DEFAULT '',
`ChangeUuid` char(36) DEFAULT NULL,
`StationUuid` char(36) DEFAULT NULL,
PRIMARY KEY (`StationID`),
UNIQUE KEY `ChangeUuid` (`ChangeUuid`),
UNIQUE KEY `StationUuid` (`StationUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#, "DROP TABLE Station;");

    migrations.add_migration("20190104_014301_CreateIPVoteCheck",
r#"CREATE TABLE `IPVoteCheck` (
`CheckID` int(11) NOT NULL AUTO_INCREMENT,
`IP` varchar(15) NOT NULL,
`StationID` int(11) NOT NULL,
`VoteTimestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
PRIMARY KEY (`CheckID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE IPVoteCheck");

    migrations.add_migration("20190104_014302_CreateLanguageCache",
r#"CREATE TABLE `LanguageCache` (
`LanguageName` varchar(150) NOT NULL,
`StationCount` int(11) DEFAULT '0',
`StationCountWorking` int(11) DEFAULT '0',
PRIMARY KEY (`LanguageName`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE LanguageCache");

    migrations.add_migration("20190104_014303_CreateTagCache",
r#"CREATE TABLE `TagCache` (
`TagName` varchar(150) NOT NULL,
`StationCount` int(11) DEFAULT '0',
`StationCountWorking` int(11) DEFAULT '0',
PRIMARY KEY (`TagName`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE TagCache");

    migrations.add_migration("20190104_014304_CreateStationCheck",
r#"CREATE TABLE `StationCheck` (
`CheckID` int(11) NOT NULL AUTO_INCREMENT,
`StationUuid` char(36) NOT NULL,
`CheckUuid` char(36) NOT NULL,
`Source` varchar(100) NOT NULL,
`Codec` varchar(20) DEFAULT NULL,
`Bitrate` int(11) NOT NULL DEFAULT '0',
`Hls` tinyint(1) NOT NULL DEFAULT '0',
`CheckOK` tinyint(1) NOT NULL DEFAULT '1',
`CheckTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
`UrlCache` text,
PRIMARY KEY (`CheckID`),
UNIQUE KEY `CheckUuid` (`CheckUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StationCheck");

    migrations.add_migration("20190104_014305_CreateStationClick",
r#"CREATE TABLE `StationClick` (
`ClickID` int(11) NOT NULL AUTO_INCREMENT,
`StationID` int(11) DEFAULT NULL,
`ClickTimestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
`IP` varchar(50) DEFAULT NULL,
PRIMARY KEY (`ClickID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StationClick");

    migrations.add_migration("20190104_014306_CreateStationHistory",
r#"CREATE TABLE `StationHistory` (
`StationChangeID` int(11) NOT NULL AUTO_INCREMENT,
`StationID` int(11) NOT NULL,
`Name` text,
`Url` text,
`Homepage` text,
`Favicon` text,
`Creation` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
`Country` varchar(50) DEFAULT NULL,
`Subcountry` varchar(50) DEFAULT NULL,
`Language` varchar(50) DEFAULT NULL,
`Tags` text,
`Votes` int(11) DEFAULT '0',
`NegativeVotes` int(11) DEFAULT '0',
`IP` varchar(50) NOT NULL DEFAULT '',
`ChangeUuid` char(36) DEFAULT NULL,
`StationUuid` char(36) DEFAULT NULL,
PRIMARY KEY (`StationChangeID`),
UNIQUE KEY `ChangeUuid` (`ChangeUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StationHistory");

    migrations.add_migration("20190104_014307_CreatePullServers",
r#"CREATE TABLE PullServers (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name TEXT NOT NULL,
    lastid TEXT,
    lastcheckid TEXT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#, "DROP TABLE PullServers;");

    migrations.add_migration("20190104_014304_CreateStationCheckHistory",
r#"CREATE TABLE `StationCheckHistory` (
`CheckID` int(11) NOT NULL AUTO_INCREMENT,
`StationUuid` char(36) NOT NULL,
`CheckUuid` char(36) NOT NULL,
`Source` varchar(100) NOT NULL,
`Codec` varchar(20) DEFAULT NULL,
`Bitrate` int(11) NOT NULL DEFAULT '0',
`Hls` tinyint(1) NOT NULL DEFAULT '0',
`CheckOK` tinyint(1) NOT NULL DEFAULT '1',
`CheckTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
`InsertTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
`UrlCache` text,
PRIMARY KEY (`CheckID`),
UNIQUE KEY `CheckUuid` (`CheckUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StationCheckHistory");

    migrations.add_migration("20190816_010900_AddStationCountryCode",
r#"ALTER TABLE `Station` ADD COLUMN CountryCode varchar(2)"#, 
r#"ALTER TABLE `Station` DROP COLUMN CountryCode"#);

    migrations.add_migration("20190816_010900_AddStationHistoryCountryCode",
r#"ALTER TABLE `StationHistory` ADD COLUMN CountryCode varchar(2)"#, 
r#"ALTER TABLE `StationHistory` DROP COLUMN CountryCode"#);

    migrations.add_migration("20191211_210000_Remove_Station_NegativeVotes",
r#"ALTER TABLE `Station` DROP COLUMN NegativeVotes"#,
r#"ALTER TABLE `Station` ADD COLUMN NegativeVotes int(11) DEFAULT '0'"#);

    migrations.add_migration("20191211_210500_Remove_StationHistory_NegativeVotes",
r#"ALTER TABLE `StationHistory` DROP COLUMN NegativeVotes"#,
r#"ALTER TABLE `StationHistory` ADD COLUMN NegativeVotes int(11) DEFAULT '0'"#);

    migrations.add_migration("20191228_123000_Remove_Station_IP",
r#"ALTER TABLE `Station` DROP COLUMN IP"#,
r#"ALTER TABLE `Station` ADD COLUMN IP varchar(50) NOT NULL DEFAULT ''"#);

    migrations.add_migration("20191228_123200_Remove_StationHistory_IP",
r#"ALTER TABLE `StationHistory` DROP COLUMN IP"#,
r#"ALTER TABLE `StationHistory` ADD COLUMN IP varchar(50) NOT NULL DEFAULT ''"#);

    migrations.add_migration("20200101_160000_Add_Station_LastLocalCheckTime",
r#"ALTER TABLE `Station` ADD COLUMN LastLocalCheckTime DATETIME"#,
r#"ALTER TABLE `Station` DROP COLUMN LastLocalCheckTime"#);

    migrations.add_migration("20200105_150500_Modify_StationHistory_StationUuid_NotNull",
r#"ALTER TABLE `StationHistory` MODIFY COLUMN StationUuid CHAR(36) NOT NULL"#,
r#"ALTER TABLE `StationHistory` MODIFY COLUMN StationUuid CHAR(36)"#);

    migrations.add_migration("20200105_150501_Modify_StationHistory_ChangeUuid_NotNull",
r#"ALTER TABLE `StationHistory` MODIFY COLUMN ChangeUuid CHAR(36) NOT NULL"#,
r#"ALTER TABLE `StationHistory` MODIFY COLUMN ChangeUuid CHAR(36)"#);

    migrations.add_migration("20200106_004000_Add_StationCheck_Icy_Info",
r#"ALTER TABLE `StationCheck` ADD COLUMN MetainfoOverridesDatabase BOOL NOT NULL DEFAULT false,
ADD COLUMN Public BOOL,
ADD COLUMN Name TEXT,
ADD COLUMN Description TEXT,
ADD COLUMN Tags TEXT,
ADD COLUMN CountryCode TEXT,
ADD COLUMN Homepage TEXT,
ADD COLUMN Favicon TEXT,
ADD COLUMN Loadbalancer TEXT"#,
r#"ALTER TABLE `StationCheck` DROP COLUMN MetainfoOverridesDatabase,
DROP COLUMN Public,
DROP COLUMN Name,
DROP COLUMN Description,
DROP COLUMN Tags,
DROP COLUMN CountryCode,
DROP COLUMN Homepage,
DROP COLUMN Favicon,
DROP COLUMN Loadbalancer
"#);

migrations.add_migration("20200106_123000_Add_StationCheckHistory_Icy_Info",
r#"ALTER TABLE `StationCheckHistory` ADD COLUMN MetainfoOverridesDatabase BOOL NOT NULL DEFAULT false,
ADD COLUMN Public BOOL,
ADD COLUMN Name TEXT,
ADD COLUMN Description TEXT,
ADD COLUMN Tags TEXT,
ADD COLUMN CountryCode TEXT,
ADD COLUMN Homepage TEXT,
ADD COLUMN Favicon TEXT,
ADD COLUMN Loadbalancer TEXT"#,
r#"ALTER TABLE `StationCheckHistory` DROP COLUMN MetainfoOverridesDatabase,
DROP COLUMN Public,
DROP COLUMN Name,
DROP COLUMN Description,
DROP COLUMN Tags,
DROP COLUMN CountryCode,
DROP COLUMN Homepage,
DROP COLUMN Favicon,
DROP COLUMN Loadbalancer
"#);

migrations.add_migration("20200110_225600_Add_FK_StationCheck_Station",
r#"ALTER TABLE StationCheck ADD CONSTRAINT FK_StationCheck_Station FOREIGN KEY (StationUuid) REFERENCES Station(StationUuid);"#,
r#"ALTER TABLE StationCheck DROP FOREIGN KEY FK_StationCheck_Station;"#);

migrations.add_migration("20200110_233000_Add_IN_StationCheck_StationUuid_Source",
r#"CREATE UNIQUE INDEX IN_StationCheck_Station ON StationCheck(StationUuid,Source);"#,
r#"DROP INDEX IN_StationCheck_Station ON StationCheck;"#);

    Ok(migrations)
}