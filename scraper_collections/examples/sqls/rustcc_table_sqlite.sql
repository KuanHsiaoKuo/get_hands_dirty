-- DROP TABLE IF EXISTS `daily_page`;
CREATE TABLE IF NOT EXISTS `daily_page`
(
    `id`           INTEGER PRIMARY KEY AUTOINCREMENT,
    `title`        TEXT DEFAULT NULL,
    `url`          TEXT DEFAULT NULL,
    `publish_date` TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS `daily_page_content`
(
    `id`           INTEGER PRIMARY KEY AUTOINCREMENT,
    `title`        TEXT    DEFAULT NULL,
    `md_content`   TEXT    DEFAULT NULL,
    `publish_page` INTEGER DEFAULT NULL,
    `tags` TEXT DEFAULT NULL
);

-- INSERT INTO `daily_page`
-- VALUES (1, '标题1', 'http://xxx', '11-234-');
