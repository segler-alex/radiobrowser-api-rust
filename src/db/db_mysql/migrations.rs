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

migrations.add_migration("20200111_155000_Add_StationClick_Uuids",
r#"ALTER TABLE StationClick ADD COLUMN StationUuid CHAR(36), ADD COLUMN ClickUuid CHAR(36);"#,
r#"ALTER TABLE StationClick DROP COLUMN StationUuid, DROP COLUMN ClickUuid;"#);

migrations.add_migration("20200111_155500_Add_IN_StationClick_ClickUuid",
r#"ALTER TABLE StationClick ADD CONSTRAINT IN_ClickUuid UNIQUE INDEX(ClickUuid);"#,
r#"ALTER TABLE StationClick DROP CONSTRAINT IN_ClickUuid;"#);

migrations.add_migration("20200111_162100_Add_PullServers_ClickUuid",
r#"ALTER TABLE PullServers ADD COLUMN lastclickuuid TEXT;"#,
r#"ALTER TABLE PullServers DROP COLUMN lastclickuuid;"#);

migrations.add_migration("20200111_191500_Add_FK_StationClick_Station",
r#"ALTER TABLE StationClick ADD CONSTRAINT FK_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid);"#,
r#"ALTER TABLE StationClick DROP CONSTRAINT FK_Station;"#);

migrations.add_migration("20200111_192500_Modify_Station_clickcount_notnull",
r#"ALTER TABLE Station MODIFY clickcount INT NOT NULL DEFAULT 0;"#,
r#"ALTER TABLE Station MODIFY clickcount INT DEFAULT 0;"#);

migrations.add_migration("20200111_204500_Modify_StationClick_clicktimestamp",
r#"ALTER TABLE StationClick MODIFY ClickTimestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;"#,
r#"ALTER TABLE StationClick MODIFY ClickTimestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;"#);

migrations.add_migration("20200111_204600_Modify_StationCheck_checktime",
r#"ALTER TABLE StationCheck MODIFY CheckTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;"#,
r#"ALTER TABLE StationCheck MODIFY CheckTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;"#);

migrations.add_migration("20200111_204700_Add_StationClick_inserttime",
r#"ALTER TABLE StationClick ADD COLUMN InsertTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;"#,
r#"ALTER TABLE StationClick DROP COLUMN InsertTime;"#);

migrations.add_migration("20200111_204800_Add_StationCheck_inserttime",
r#"ALTER TABLE StationCheck ADD COLUMN InsertTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;"#,
r#"ALTER TABLE StationCheck DROP COLUMN InsertTime;"#);

migrations.add_migration("20200112_121800_Remove_StationClick_StationID",
r#"ALTER TABLE StationClick DROP COLUMN StationID;"#,
r#"ALTER TABLE StationClick ADD COLUMN StationID INT;"#);

    migrations.add_migration("20200113_202000_Modify_Station_StationID_bigint",
r#"ALTER TABLE Station MODIFY COLUMN StationID BIGINT UNSIGNED NOT NULL AUTO_INCREMENT;"#,
r#"ALTER TABLE Station MODIFY COLUMN StationID INT NOT NULL AUTO_INCREMENT;"#);

    migrations.add_migration("20200113_202100_Modify_StationHistory_StationChangeID_bigint",
r#"ALTER TABLE StationHistory MODIFY COLUMN StationChangeID BIGINT UNSIGNED NOT NULL AUTO_INCREMENT;"#,
r#"ALTER TABLE StationHistory MODIFY COLUMN StationChangeID INT NOT NULL AUTO_INCREMENT;"#);

    migrations.add_migration("20200113_202200_Modify_StationCheck_CheckID_bigint",
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CheckID BIGINT UNSIGNED NOT NULL AUTO_INCREMENT;"#,
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CheckID INT NOT NULL AUTO_INCREMENT;"#);

    migrations.add_migration("20200113_202300_Modify_StationClick_ClickID_bigint",
r#"ALTER TABLE StationClick MODIFY COLUMN ClickID BIGINT UNSIGNED NOT NULL AUTO_INCREMENT;"#,
r#"ALTER TABLE StationClick MODIFY COLUMN ClickID INT NOT NULL AUTO_INCREMENT;"#);

    migrations.add_migration("20200113_203500_Add_FK_StationCheckHistory_Station",
r#"ALTER TABLE StationCheckHistory ADD CONSTRAINT FK_StationCheckHistory_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid);"#,
r#"ALTER TABLE StationCheckHistory DROP CONSTRAINT FK_StationCheckHistory_Station;"#);

    migrations.add_migration("20200113_203600_Drop_StationHistory_StationID",
r#"ALTER TABLE StationHistory DROP COLUMN StationID;"#,
r#"ALTER TABLE StationHistory ADD COLUMN StationID INT NOT NULL;"#);

    migrations.add_migration("20200113_203700_Drop_StationCheck",
r#"DROP TABLE StationCheck;"#,
r#"CREATE TABLE StationCheck(
CheckID INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
StationUuid CHAR(36) NOT NULL,
CheckUuid CHAR(36) NOT NULL UNIQUE,
Source VARCHAR(100) NOT NULL,
Codec VARCHAR(20),
Bitrate INT NOT NULL DEFAULT 0,
Hls BOOLEAN NOT NULL DEFAULT FALSE,
CheckOK  BOOLEAN NOT NULL DEFAULT TRUE,
CheckTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
UrlCache TEXT,
MetainfoOverridesDatabase BOOL NOT NULL DEFAULT false,
Public BOOL,
Name TEXT,
Description TEXT,
Tags TEXT,
CountryCode TEXT,
Homepage TEXT,
Favicon TEXT,
Loadbalancer TEXT,
InsertTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#);

    migrations.add_migration("20200113_204000_Create_View_StationCheck",
r#"CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck;"#);

    migrations.add_migration("20200114_001000_Delete_StationCheckHistory",
r#"DELETE FROM StationCheckHistory;"#,
r#"DELETE FROM StationCheckHistory;"#);

    migrations.add_migration("20200114_001100_Delete_PullServers",
r#"DELETE FROM PullServers;"#,
r#"DELETE FROM PullServers;"#);

    migrations.add_migration("20200118_135000_Modify_Station_Creation",
r#"ALTER TABLE Station MODIFY COLUMN Creation DATETIME NOT NULL;"#,
r#"ALTER TABLE Station MODIFY COLUMN Creation TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;"#);

    migrations.add_migration("20200118_135500_Modify_StationHistory_Creation",
r#"ALTER TABLE StationHistory MODIFY COLUMN Creation DATETIME NOT NULL;"#,
r#"ALTER TABLE StationHistory MODIFY COLUMN Creation TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;"#);

    migrations.add_migration("20200118_135600_Modify_IPVoteCheck_VoteTimestamp",
r#"ALTER TABLE IPVoteCheck MODIFY COLUMN VoteTimestamp DATETIME NOT NULL;"#,
r#"ALTER TABLE IPVoteCheck MODIFY COLUMN VoteTimestamp TIMESTAMP NOT NULL;"#);

    migrations.add_migration("20200118_135700_Modify_StationClick_ClickTimestamp",
r#"ALTER TABLE StationClick MODIFY COLUMN ClickTimestamp DATETIME NOT NULL;"#,
r#"ALTER TABLE StationClick MODIFY COLUMN ClickTimestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;"#);

    migrations.add_migration("20200118_135800_Modify_StationClick_InsertTime",
r#"ALTER TABLE StationClick MODIFY COLUMN InsertTime DATETIME NOT NULL;"#,
r#"ALTER TABLE StationClick MODIFY COLUMN InsertTime TIMESTAMP NOT NULL;"#);

    migrations.add_migration("20200118_135900_Modify_StationCheckHistory_CheckTime",
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CheckTime DATETIME NOT NULL;"#,
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CheckTime TIMESTAMP NOT NULL;"#);

    migrations.add_migration("20200118_135930_Modify_StationCheckHistory_InsertTime",
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN InsertTime DATETIME NOT NULL;"#,
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN InsertTime TIMESTAMP NOT NULL;"#);

    migrations.add_migration("20200119_121000_Drop_FK_StationCheckHistory_Station",
r#"ALTER TABLE StationCheckHistory DROP FOREIGN KEY FK_StationCheckHistory_Station;"#,
r#"ALTER TABLE StationCheckHistory ADD CONSTRAINT FK_StationCheckHistory_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid);"#);

    migrations.add_migration("20200119_121100_Add_FK_StationCheckHistory_Station",
r#"ALTER TABLE StationCheckHistory ADD CONSTRAINT FK_StationCheckHistory_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid) ON DELETE CASCADE;"#,
r#"ALTER TABLE StationCheckHistory DROP FOREIGN KEY FK_StationCheckHistory_Station;"#);

    migrations.add_migration("20200119_121200_Drop_FK_StationClick_Station",
r#"ALTER TABLE StationClick DROP FOREIGN KEY FK_Station;"#,
r#"ALTER TABLE StationClick ADD CONSTRAINT FK_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid);"#);

    migrations.add_migration("20200119_121300_Add_FK_StationClick_Station",
r#"ALTER TABLE StationClick ADD CONSTRAINT FK_StationClick_Station FOREIGN KEY(StationUuid) REFERENCES Station(StationUuid) ON DELETE CASCADE;"#,
r#"ALTER TABLE StationClick DROP FOREIGN KEY FK_StationClick_Station;"#);

    migrations.add_migration("20200202_003500_Add_Index_StationCheckHistory_StationUuid_Source",
r#"ALTER TABLE StationCheckHistory ADD INDEX IN_StationCheckHistory_StationUuid_Source(StationUuid,Source);"#,
r#"ALTER TABLE StationCheckHistory DROP INDEX IN_StationCheckHistory_StationUuid_Source;"#);

    migrations.add_migration("20200202_003700_Add_Index_StationCheckHistory_InsertTime",
r#"ALTER TABLE StationCheckHistory ADD INDEX IN_StationCheckHistory_InsertTime(InsertTime);"#,
r#"ALTER TABLE StationCheckHistory DROP INDEX IN_StationCheckHistory_InsertTime;"#);

    migrations.add_migration("20200202_012500_Add_Index_StationClick_StationUuid_ClickTimestamp",
r#"ALTER TABLE StationClick ADD INDEX IN_StationClick_StationUuid_ClickTimestamp(StationUuid, ClickTimestamp);"#,
r#"ALTER TABLE StationClick DROP INDEX IN_StationClick_StationUuid_ClickTimestamp;"#);

    migrations.add_migration("20200526_211500_Modify_IPVoteCheck_IP_IPv6",
r#"ALTER TABLE IPVoteCheck MODIFY COLUMN IP VARCHAR(50) NOT NULL;"#,
r#"ALTER TABLE IPVoteCheck MODIFY COLUMN IP VARCHAR(15) NOT NULL;"#);

    migrations.add_migration("20201118_204500_Modify_Station_Country",
r#"ALTER TABLE Station MODIFY COLUMN Country VARCHAR(250);"#,
r#"ALTER TABLE Station MODIFY COLUMN Country VARCHAR(50);"#);

    migrations.add_migration("20201118_205000_Drop_StationHistory_Country",
r#"ALTER TABLE StationHistory DROP COLUMN Country;"#,
r#"ALTER TABLE StationHistory ADD COLUMN Country VARCHAR(50);"#);

    migrations.add_migration("20201123_220500_Add_Station_CountrySubdivisionCode",
r#"ALTER TABLE Station ADD COLUMN CountrySubdivisionCode VARCHAR(3) NULL;"#,
r#"ALTER TABLE Station DROP COLUMN CountrySubdivisionCode;"#);

    migrations.add_migration("20201123_221000_Add_StationCheckHistory_CountrySubdivisionCode",
r#"ALTER TABLE StationCheckHistory ADD COLUMN CountrySubdivisionCode VARCHAR(3) NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN CountrySubdivisionCode;"#);

    migrations.add_migration("20210101_233000_Add_StationCheckHistory_DoNotIndex",
r#"ALTER TABLE StationCheckHistory ADD COLUMN DoNotIndex BOOLEAN NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN DoNotIndex;"#);

    migrations.add_migration("20210101_233500_Recreate_View_StationCheck",
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#);

    migrations.add_migration("20210406_230000_Add_StationCheckHistory_ServerSoftware",
r#"ALTER TABLE StationCheckHistory ADD COLUMN ServerSoftware TEXT NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN ServerSoftware;"#);

    migrations.add_migration("20210406_230001_Add_StationCheckHistory_Sampling",
r#"ALTER TABLE StationCheckHistory ADD COLUMN Sampling INT UNSIGNED NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN Sampling;"#);

    migrations.add_migration("20210406_230002_Add_StationCheckHistory_LanguageCodes",
r#"ALTER TABLE StationCheckHistory ADD COLUMN LanguageCodes TEXT NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN LanguageCodes;"#);

    migrations.add_migration("20210406_230003_Add_StationCheckHistory_TimingMs",
r#"ALTER TABLE StationCheckHistory ADD COLUMN TimingMs INT UNSIGNED NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN TimingMs;"#);

    migrations.add_migration("20210406_233500_Recreate_View_StationCheck",
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex,CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#);

    migrations.add_migration("20210409_190003_Add_StationCheckHistory_SslError",
r#"ALTER TABLE StationCheckHistory ADD COLUMN SslError BOOLEAN NOT NULL DEFAULT FALSE;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN SslError;"#);

    migrations.add_migration("20210409_190010_Recreate_View_StationCheck",
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex,CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs,SslError FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex,CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#);

    migrations.add_migration("20210412_231000_CreateStationCheckStep",
r#"CREATE TABLE `StationCheckStep` (
`Id` int(11) NOT NULL AUTO_INCREMENT,
`StepUuid` char(36) NOT NULL,
`ParentStepUuid` char(36) DEFAULT NULL,
`CheckUuid` char(36) NOT NULL,
`StationUuid` char(36) NOT NULL,
`Url` text NOT NULL,
`UrlType` text DEFAULT NULL,
`Error` text DEFAULT NULL,
`InsertTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
PRIMARY KEY (`Id`),
UNIQUE KEY `StepUuid` (`StepUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StationCheckStep");

    migrations.add_migration("20210413_211000_Add_FK_StationCheckStep_StationCheckHistory",
r#"ALTER TABLE StationCheckStep ADD CONSTRAINT FK_StationCheckStep_StationCheckHistory FOREIGN KEY(CheckUuid) REFERENCES StationCheckHistory(CheckUuid) ON DELETE CASCADE;"#,
r#"ALTER TABLE StationCheckStep DROP CONSTRAINT FK_StationCheckStep_StationCheckHistory;"#);

    migrations.add_migration("20210414_190003_Add_Station_GeoLat",
r#"ALTER TABLE Station ADD COLUMN GeoLat DOUBLE NULL;"#,
r#"ALTER TABLE Station DROP COLUMN GeoLat;"#);

    migrations.add_migration("20210414_190004_Add_Station_GeoLong",
r#"ALTER TABLE Station ADD COLUMN GeoLong DOUBLE NULL;"#,
r#"ALTER TABLE Station DROP COLUMN GeoLong;"#);

    migrations.add_migration("20210414_190005_Add_Station_GeoLat",
r#"ALTER TABLE StationCheckHistory ADD COLUMN GeoLat DOUBLE NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN GeoLat;"#);

    migrations.add_migration("20210414_190006_Add_Station_GeoLong",
r#"ALTER TABLE StationCheckHistory ADD COLUMN GeoLong DOUBLE NULL;"#,
r#"ALTER TABLE StationCheckHistory DROP COLUMN GeoLong;"#);

    migrations.add_migration("20210414_190007_Add_StationHistory_GeoLat",
r#"ALTER TABLE StationHistory ADD COLUMN GeoLat DOUBLE NULL;"#,
r#"ALTER TABLE StationHistory DROP COLUMN GeoLat;"#);

    migrations.add_migration("20210414_190008_Add_StationHistory_GeoLong",
r#"ALTER TABLE StationHistory ADD COLUMN GeoLong DOUBLE NULL;"#,
r#"ALTER TABLE StationHistory DROP COLUMN GeoLong;"#);

    migrations.add_migration("20210414_190009_Add_Station_SslError",
r#"ALTER TABLE Station ADD COLUMN SslError BOOLEAN NOT NULL DEFAULT FALSE;"#,
r#"ALTER TABLE Station DROP COLUMN SslError;"#);

    migrations.add_migration("20210414_190010_Recreate_View_StationCheck",
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT * FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex,CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs,SslError FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#);

    migrations.add_migration("20210414_210007_Add_Station_LanguageCodes",
r#"ALTER TABLE Station ADD COLUMN LanguageCodes TEXT NULL;"#,
r#"ALTER TABLE Station DROP COLUMN LanguageCodes;"#);

    migrations.add_migration("20210414_210008_Add_StationHistory_LanguageCodes",
r#"ALTER TABLE StationHistory ADD COLUMN LanguageCodes TEXT NULL;"#,
r#"ALTER TABLE StationHistory DROP COLUMN LanguageCodes;"#);

    migrations.add_migration("20210708_211807_Add_Station_ExtendedInfo",
r#"ALTER TABLE Station ADD COLUMN ExtendedInfo BOOLEAN NOT NULL DEFAULT FALSE;"#,
r#"ALTER TABLE Station DROP COLUMN ExtendedInfo;"#);

    migrations.add_migration("20210905_214000_Change_Station_CountrySubdivisionCode",
r#"ALTER TABLE Station MODIFY COLUMN CountrySubdivisionCode VARCHAR(6) NULL;"#,
r#"ALTER TABLE Station MODIFY COLUMN CountrySubdivisionCode VARCHAR(3) NULL;"#);

    migrations.add_migration("20210905_214001_Change_StationCheckHistory_CountrySubdivisionCode",
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CountrySubdivisionCode VARCHAR(6) NULL;"#,
r#"ALTER TABLE StationCheckHistory MODIFY COLUMN CountrySubdivisionCode VARCHAR(3) NULL;"#);

    migrations.add_migration("20210905_215000_Recreate_View_StationCheck",
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT * FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#,
r#"DROP VIEW StationCheck; CREATE VIEW StationCheck AS SELECT CheckID,CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime,DoNotIndex,CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs,SslError FROM StationCheckHistory WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);"#);

    migrations.add_migration("20211008_203000_Create_Table_StreamingServers",
r#"CREATE TABLE `StreamingServers` (
`Id` INT(11) NOT NULL AUTO_INCREMENT,
`Uuid` CHAR(36) NOT NULL,
`Url` VARCHAR(300) NOT NULL,
`StatusUrl` TEXT,
`Status` JSON,
`Error` VARCHAR(50),
`CreatedAt` DATETIME NOT NULL,
`UpdatedAt` DATETIME,
PRIMARY KEY (`Id`),
UNIQUE KEY `Uuid` (`Uuid`),
UNIQUE KEY `Url` (`Url`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE StreamingServers");

    migrations.add_migration("20211010_171000_Add_Station_StreamingServers",
r#"ALTER TABLE Station ADD COLUMN ServerUuid CHAR(36);"#,
r#"ALTER TABLE Station DROP COLUMN ServerUuid;"#);

    Ok(migrations)
}
