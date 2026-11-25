create view EUTransactionType
as
select top 100000
 EUTransactionTypeId, EUTransactionTypeCode, EUTransactionTypeNameEN as EUTransactionTypeName,
 Level, ParentEUTransactionTypeId
from [EUTransactionTypeDictionary] (nolock)
order by IsInactive , EUTransactionTypeCode
