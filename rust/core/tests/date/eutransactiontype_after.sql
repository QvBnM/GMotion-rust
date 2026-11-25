CREATE TABLE [dbo].[EUTransactionType] (
    [EUTransactionTypeId] int NOT NULL,
    [EUTransactionTypeCode] varchar(25) NOT NULL,
    [EUTransactionTypeName] varchar(255) NOT NULL,
    [Level] int NOT NULL,
    [ParentEUTransactionTypeId] int NULL
)
