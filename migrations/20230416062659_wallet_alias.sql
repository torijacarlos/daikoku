-- Add migration script here
ALTER TABLE DAIKOKU.WALLET 
ADD alias VARCHAR(255) NOT NULL DEFAULT "Wallet";
