-- query.sql

-- name: GetAdmins :one
SELECT admins FROM config;


-- name: AddAdmin :exec
UPDATE  config
SET     admins = json_set(admins, '$[#]', ?1);

-- name: SetAdmins :exec
UPDATE config SET admins = ?1;

-- name: GetTreasuryNotificationsChatID :one
SELECT treasury_notifications_chat_id FROM config;

-- name: SetTreasuryNotificationsChatID :exec
UPDATE config SET treasury_notifications_chat_id = ?1;

-- name: GetAssemblyMinutesChatID :one
SELECT assembly_minutes_chat_id FROM config;

-- name: SetAssemblyMinutesChatID :exec
UPDATE config SET assembly_minutes_chat_id = ?1;

-- name: GetFestAttendees :many
SELECT * FROM festAttendee;

-- name: HasPaidFest :one
SELECT hasPaid FROM festAttendee WHERE codeName = ?1;

-- name: LegalNameSetPaid :exec
UPDATE festAttendee SET hasPaid = ?2 WHERE legalName = ?1;

-- name: NewFestAttendee :exec
INSERT INTO festAttendee (legalName, codeName, assistsTo, allergies, diet) VALUES (?1, ?2, ?3, ?4, ?5);

-- name: GetAttendessNames :many
SELECT codeName, legalName from festAttendee;

-- name: WhoGoesToSundal :many
SELECT codeName FROM festAttendee a WHERE EXISTS (
    SELECT 1
    FROM json_each(a.assistsTo) AS place
    WHERE place.value = 'sundal'
) ORDER BY codeName;


-- name: HowManyGoToSundal :one
SELECT COUNT(*) FROM festAttendee a WHERE EXISTS (
    SELECT 1
    FROM json_each(a.assistsTo) AS place
    WHERE place.value = 'sundal'
);

-- name: HowManyGoToBubu :one
SELECT COUNT(*) FROM festAttendee a WHERE EXISTS (
    SELECT 1
    FROM json_each(a.assistsTo) AS place
    WHERE place.value = 'bubu'
);

-- name: HowManyGoToAlmuerzo :one
SELECT COUNT(*) FROM festAttendee a WHERE EXISTS (
    SELECT 1
    FROM json_each(a.assistsTo) AS place
    WHERE place.value = 'sundal'
);

-- name: HowManyStandardDiet :one
SELECT COUNT(*) FROM festAttendee WHERE diet = 0 AND allergies IS NULL;

-- name: WhoVegetarian :many
SELECT codeName FROM festAttendee WHERE diet = 1;

-- name: WhoVegan :many
SELECT codeName FROM festAttendee WHERE diet = 2;

-- name: WhoAllergies :many
SELECT codeName, allergies FROM festAttendee WHERE allergies IS NOT NULL;

-- Here the % {number} is the max number of slots we can have, right now is set to
-- a max of 5
-- name: RotatePointer :one
INSERT INTO historyPointers (userId, lastSlot)
VALUES (?1, 1)
ON CONFLICT(userId) DO UPDATE SET
    lastSlot = (historyPointers.lastSlot % 5) + 1
RETURNING lastSlot;

-- name: SaveHistory :exec
INSERT INTO userHistory (userId, slotId, message, answer, createdAt)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(userId, slotId) DO UPDATE SET
    message = excluded.message,
    answer = excluded.answer,
    createdAt = excluded.createdAt;

-- name: GetHistory :many
SELECT message, answer FROM userHistory WHERE userId = ?1 AND answer != '' ORDER BY createdAt ASC;

-- Treasury updates queries

-- name: GetUnannouncedTreasuryUpdates :many
SELECT id, description, amount, createdAt FROM treasuryUpdates WHERE announced = 0 ORDER BY createdAt ASC;

-- name: GetTreasuryUpdatesInPeriod :many
SELECT id, description, amount, createdAt, announced FROM treasuryUpdates WHERE createdAt >= ?1 AND createdAt <= ?2 ORDER BY createdAt ASC;

-- name: AddTreasuryUpdate :one
INSERT INTO treasuryUpdates (description, amount, createdAt, announced) VALUES (?1, ?2, ?3, 0) RETURNING id;

-- name: SetTreasuryUpdateAnnounced :exec
UPDATE treasuryUpdates SET announced = 1 WHERE id = ?1;

-- name: GetAllAssociateEmails :many
SELECT email FROM associates;

-- name: AddAssociate :one
INSERT INTO associates (nickName, email) VALUES (?1, ?2) ON CONFLICT(nickName) DO NOTHING RETURNING nickName;

-- name: RemoveAssociate :exec
DELETE FROM associates WHERE nickName = ?1;

-- name: ListAssociates :many
SELECT nickName, email FROM associates;

-- name: GetAssociate :one
SELECT nickName, email FROM associates WHERE nickName = ?1;

-- name: UpdateAssociate :exec
UPDATE associates SET nickName = ?1, email = ?2 WHERE nickName = ?3;
