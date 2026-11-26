/************************************************************************ Documentation header ***
Module: [Stocuri 2]

Summary: [se cheama din Stock_StockProcessDispatch dupa terminarea procesarii]

*** Documentation header *************************************************************************/

CREATE PROCEDURE [dbo].[Stock_StockProcessDispatchAfterProcess_UpdateItemCost]
  @SystemId INT,
  @WorkingSiteId INT,
  @WorkingStrategy INT,
  @FirstEventBMoment DATETIME,
  @FirstEventTimeStamp DATETIME,
  @FirstEventId INT,
  @LastEventBMoment DATETIME,
  @LastEventTimeStamp DATETIME,
  @LastEventId INT,
  @GUID UNIQUEIDENTIFIER
AS
BEGIN
DECLARE @STD INT
DECLARE @FIFO INT
DECLARE @LIFO INT
DECLARE @DI INT
DECLARE @WAC INT
DECLARE @mWAC INT
DECLARE @gWAC INT

SET @STD = 1
SET @FIFO = 2
SET @LIFO = 4
SET @DI = 8
SET @WAC = 16
SET @mWAC = 32
SET @gWAC = 64

-------------- Operatii diferentiate pe tip de procesare---------------
--  IF @WorkingStrategy & @FIFO = @FIFO
--  IF @WorkingStrategy & @LIFO = @LIFO
--  IF @WorkingStrategy & @DI = @DI
--  IF @WorkingStrategy & @WAC = @WAC
--  IF @WorkingStrategy & @mWAC = @mWAC
--  IF @WorkingStrategy & @gWAC = @gWAC

-------------- Operatii standard ---------------


SELECT E.DocumentDetailId, E.SiteId, abs(sum(JournalPrice*JournalQtty)) as  CostValue,
COALESCE(CAST(abs(sum(JournalPriceC1*JournalQtty))as decimal(19,2)),0) as CostValue1 ,
COALESCE(CAST(abs(sum(JournalPriceC2*JournalQtty))as decimal(19,2)),0) as CostValue2 ,
COALESCE(CAST(abs(sum(JournalPriceC3*JournalQtty))as decimal(19,2)),0) as CostValue3 ,
COALESCE(CAST(abs(sum(JournalPriceC4*JournalQtty))as decimal(19,2)),0) as CostValue4 ,
CAST((sum(JournalPrice*abs(JournalQtty)) / sum(abs(JournalQtty)))as decimal(19,4)) as EventUnitPrice , E.EventId  , d.DocumentTypeId, d.DocumentId
INTO #COSTVALUE
FROM vwAllStockState FF (NOLOCK)
JOIN Event E (NOLOCK) on E.EventId=FF.CauseEventId AND E.SiteId=FF.CauseSiteId
join DocumentDetail dd (nolock) on E.DocumentDetailId=dd.DocumentDetailId and E.SiteId=dd.SiteId
join Document d (nolock) on dd.DocumentId=d.DocumentId and dd.SiteId=d.SiteId
JOIN Site S (nolock) on S.SiteId = d.SiteId
WHERE E.DocumentDetailId is not null
AND S.StockProcessingSiteId = @WorkingSiteId
AND E.BMoment BETWEEN @FirstEventBMoment AND @LastEventBMoment
GROUP BY E.DocumentDetailId, E.SiteId  , E.EventId , d.DocumentTypeId, d.DocumentId

--WHERE E.DocumentDetailId is not null
--AND E.BMoment BETWEEN @FirstEventBMoment AND @LastEventBMoment
--GROUP BY E.DocumentDetailId, E.SiteId  , E.EventId , d.DocumentTypeId, d.DocumentId

delete E
from #COSTVALUE E
where exists (SELECT 1 FROM Transfer T (nolock) WHERE T.EventId=E.EventId AND T.SiteId=E.SiteId )

insert into #COSTVALUE
SELECT E.DocumentDetailId, E.SiteId, CAST(abs(sum(JournalPrice*JournalQtty))as decimal(19,2)) as CostValue ,
COALESCE(CAST(abs(sum(JournalPriceC1*JournalQtty))as decimal(19,2)),0) as CostValue1 ,
COALESCE(CAST(abs(sum(JournalPriceC2*JournalQtty))as decimal(19,2)),0) as CostValue2 ,
COALESCE(CAST(abs(sum(JournalPriceC3*JournalQtty))as decimal(19,2)),0) as CostValue3 ,
COALESCE(CAST(abs(sum(JournalPriceC4*JournalQtty))as decimal(19,2)),0) as CostValue4 ,
CAST((sum(JournalPrice*abs(JournalQtty)) / sum(abs(JournalQtty)))as decimal(19,4)) as EventUnitPrice , E.EventId  , d.DocumentTypeId, d.DocumentId
FROM vwAllStockState FF (nolock)
JOIN Event E (nolock) on E.EventId=FF.CauseEventId AND E.SiteId=FF.CauseSiteId
JOIN Transfer T (nolock) ON T.EventId=E.EventId AND T.SiteId=E.SiteId
join Site s (nolock)on s.SiteId=E.SiteId
join DocumentDetail dd (nolock) on E.DocumentDetailId=dd.DocumentDetailId and E.SiteId=dd.SiteId   join Document d (nolock) on dd.DocumentId=d.DocumentId and dd.SiteId=d.SiteId
WHERE E.DocumentDetailId is not null
AND E.BMoment BETWEEN @FirstEventBMoment AND @LastEventBMoment and FF.JournalQtty>0
and s.StockProcessingSiteId=@WorkingSiteId
GROUP BY E.DocumentDetailId, E.EventId, E.SiteId , d.DocumentTypeId, d.DocumentId


UPDATE DD SET DD.ItemCost=T.CostValue
from DocumentDetail DD
 join #COSTVALUE T on DD.DocumentDetailId=T.DocumentDetailId and DD.SiteId=T.SiteId

/*Update pentru extensiile de raportare*/
UPDATE DD SET DD.ItemCost=T.CostValue,
 DD.LegalCurrencyItemCost = T.CostValue,
 DD.C1_ItemCost =  CostValue1,
 DD.C2_ItemCost =  CostValue2,
 DD.C3_ItemCost =  CostValue3,
 DD.C4_ItemCost =  CostValue4
FROM MapSalesInvoiceDetail DD
join #COSTVALUE T on DD.DocumentDetailId=T.DocumentDetailId and DD.SiteId=T.SiteId

UPDATE DD SET DD.ItemCost=T.CostValue,
 DD.LegalCurrencyItemCost = T.CostValue,
 DD.C1_ItemCost =  CostValue1,
 DD.C2_ItemCost =  CostValue2,
 DD.C3_ItemCost =  CostValue3,
 DD.C4_ItemCost =  CostValue4
FROM MapSalesInvoiceDetail DD
JOIN #COSTVALUE T on DD.SrcDocDetailId=T.DocumentDetailId and DD.SrcSiteId=T.SiteId AND DD.SrcDocId = T.DocumentId


declare @LegalCurrencyId  as int
 , @C1CurrencyId as int
 , @C2CurrencyId as int
 , @C3CurrencyId as int
 , @C4CurrencyId as int

select  @LegalCurrencyId=Value from dbo.InitializationParam (nolock) where ParamName='LegalCurrencyId'
select
 @C1CurrencyId= ISNULL(C1,0),
 @C2CurrencyId= ISNULL(C2,0),
 @C3CurrencyId= ISNULL(C3,0),
 @C4CurrencyId= ISNULL(C4,0)
from dbo.StockCurrencies (nolock)

update  msd
set DetailItemCost= case
    when msd.RefCurrencyId= @LegalCurrencyId then msi.CostValue
 when msd.RefCurrencyId= @C1CurrencyId then msi.CostValue1
    when msd.RefCurrencyId= @C2CurrencyId then msi.CostValue2
    when msd.RefCurrencyId= @C3CurrencyId then msi.CostValue3
    when msd.RefCurrencyId= @C4CurrencyId then msi.CostValue4
    else 0 end
from MapSalesInvoiceDetailExtension msd
 inner join #COSTVALUE msi (nolock) on msd.DocumentDetailId=msi.DocumentDetailId and msd.SiteId=msi.SiteId

/*Update pentru extensiile de raportare*/

/* Aviz expeditie la distanta */
  -- detail --
  update dnd
     set UnitPrice = t.EventUnitPrice,
         Amount    = t.CostValue,
         VAT       = null
  from DeliveryNoteDetail dnd
    join #COSTVALUE t on dnd.DeliveryNoteDetailId=t.DocumentDetailId and dnd.SiteId=t.SiteId
  where t.DocumentTypeId=1014
  -- detail --
  -- header --
  update dn
     set TotalAmount=x.TotalAmount
  from DeliveryNote dn
    join (
           select DocumentId, SiteId, sum(CostValue) as TotalAmount
           from #COSTVALUE
           where DocumentTypeId=1014
           group by DocumentId, SiteId
         ) x on dn.DeliveryNoteId=x.DocumentId and dn.SiteId=x.SiteId    -- header --
/* Aviz expeditie la distanta */

/* Bon de consum */
  -- detail --
  update cnd
     set StockPrice=t.EventUnitPrice,
         Price     =t.EventUnitPrice
  from ConsumptionNoteDetail cnd
    join #COSTVALUE t on cnd.DocumentDetailId=t.DocumentDetailId and cnd.SiteId=t.SiteId
  where t.DocumentTypeId=5002
  -- detail --
  -- header --
  update cn
     set cn.TotalAmount=x.TotalAmount
  from ConsumptionNote cn
    join (
           select DocumentId, SiteId, sum(CostValue) as TotalAmount
           from #COSTVALUE
           where DocumentTypeId=5002
           group by DocumentId, SiteId
         ) x on cn.DocumentId=x.DocumentId and cn.SiteId=x.SiteId
  -- header --
/* Bon de consum */

/* Nota de intrare receptie - generat din DNE */
  -- detail --
  UPDATE grnd
     SET UnitPrice         = t.EventUnitPrice,
         ReceivedUnitPrice = t.EventUnitPrice
  FROM GoodsReceivedNoteDetail grnd
    JOIN BusinessTransaction bt (nolock) on grnd.DocumentId=bt.DestDocId and grnd.SiteId=bt.SiteId
    JOIN #COSTVALUE t on grnd.GoodsReceivedNoteDetailId=t.DocumentDetailId and grnd.SiteId=t.SiteId
   JOIN TransactionTypeStockBehaviour TT (nolock)  ON TT.TransactionTypeId = bt.TransactionTypeId
WHERE TT.HasDynamicPrice = 1
  -- detail --
  -- header --
  UPDATE grn
     SET TotalAmount=x.TotalAmount
  FROM GoodsReceivedNote grn
    join (
           SELECT t.DocumentId, t.SiteId, sum(CostValue) as TotalAmount
           FROM #COSTVALUE t
             JOIN BusinessTransaction bt (nolock) on t.DocumentId=bt.DestDocId and t.SiteId=bt.SiteId
             JOIN TransactionTypeStockBehaviour TT (nolock)  ON TT.TransactionTypeId = bt.TransactionTypeId
           WHERE TT.HasDynamicPrice = 1
           GROUP BY t.DocumentId, t.SiteId
         ) x on grn.GoodsReceivedNoteId=x.DocumentId and grn.SiteId=x.SiteId
  -- header --
  /*
    210 - Generare NIR din Aviz Intern de pe E-Mail (Agenti)
    563 - Generare NIR din Aviz la Distanta (DNE)
    570 - Generare NIR din Aviz Intern de pe E-Mail (Retail)
  */
/* Nota de intrare receptie - generat din DNE */

/* Bon de consum obiecte de inventar*/
  -- detail --
  update cnd
     set StockPrice=t.EventUnitPrice
  from TransferDocumentDetail cnd
    join #COSTVALUE t on cnd.DocumentDetailId=t.DocumentDetailId and cnd.SiteId=t.SiteId
  --where t.DocumentTypeId=5010
  -- detail --
  -- header --
  update cn
     set cn.TotalAmount=x.TotalAmount
  from TransferDocument cn
    join (
           select DocumentId, SiteId, sum(CostValue) as TotalAmount
           from #COSTVALUE
           --where DocumentTypeId=5010
           group by DocumentId, SiteId
         ) x on cn.DocumentId=x.DocumentId and cn.SiteId=x.SiteId
  -- header --
/* Bon de consum */

-- select * from TransactionType where DestDocTypeId = 3003 AND Active = 1
--RETURN

 INSERT INTO ERPContextInfo
 (SPID, InitiatorIsAccountingProcess, InsertMoment)
 SELECT @@SPID, -9 InitiatorIsAccountingProcess, GetDate() InsertMoment

declare @Err int
 exec @Err = dbo.uspCommon_ProcessDocuments null, null, null
 --end uspCommon_ProcessDocuments
 if (@Err <> 0)
 begin
  raiserror ('Stock_StockProcessDispatchAfterProcess: Nu s-a  putut efectua procesarea in batch a documentelor!', 16, 1)
  return
  end

 DELETE FROM ERPContextInfo WHERE  SPID = @@SPID

END


