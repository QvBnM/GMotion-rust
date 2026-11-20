/*
exec dbo.rptInventar_Situatie
	@EndDate = '20240101',
	@UserId = 1
*/
CREATE  PROCEDURE [dbo].[rptInventar_Situatie]
  @EndDate datetime,
  @AccSystemId int,
  @CompanyId int,
  @SiteString varchar(8000),
  @ResponsablePersonId int,
  @InventoryCommission varchar(8000),
  @UserId INT,
  @ContextSiteId int
AS

SET NOCOUNT ON
SET ANSI_WARNINGS OFF

create table #tmpNume (Nume varchar(200))

insert into #tmpNume (Nume)
select p.FirstName + ' ' + p.LastName
from Person p
join #Person tp on tp.PersonId = p.PersonId

declare  @InventoryCommissionName varchar(8000)
set  @InventoryCommissionName = ''
select @InventoryCommissionName = @InventoryCommissionName + Nume + ', ' from #tmpNume
select @InventoryCommissionName = left(@InventoryCommissionName, len(@InventoryCommissionName) - 1) where len(@InventoryCommissionName) > 0

declare @SQL Nvarchar(MAX)

set @SQL = N'select a.AssetId, a.Name, a.Value from Asset a where a.SiteId in (' + @SiteString + N') and a.EndDate <= @EndDate'

exec sp_executesql @SQL, N'@EndDate datetime', @EndDate
