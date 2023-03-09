-- DROP TABLE IF EXISTS `daily_page`;
CREATE TABLE IF NOT EXISTS `daily_page`
(
    `id`           INTEGER PRIMARY KEY AUTOINCREMENT,
    `title`        TEXT DEFAULT NULL,
    `url`          TEXT DEFAULT NULL,
    `publish_date` TEXT DEFAULT NULL
);

-- INSERT INTO `daily_page`
-- VALUES (1, '标题1', 'http://xxx', '11-234-');
