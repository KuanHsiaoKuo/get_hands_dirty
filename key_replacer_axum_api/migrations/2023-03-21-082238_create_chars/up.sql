-- Your SQL goes here
CREATE TABLE `posts`
(
    id        INTEGER AUTO_INCREMENT PRIMARY KEY,
    title     VARCHAR(255) NOT NULL,
    content TEXT         NOT NULL,
    published BOOLEAN      NOT NULL DEFAULT FALSE
);
