-- Deletion means the entry may be imported again, including older tombstones.
DELETE FROM import_records WHERE disposition = 'accepted' AND transaction_id IS NULL;
CREATE TRIGGER forget_deleted_import_evidence BEFORE DELETE ON transactions
BEGIN
    DELETE FROM import_records WHERE transaction_id = OLD.id;
END;
