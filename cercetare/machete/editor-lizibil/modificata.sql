/*
exec dbo.rptInventar_Situatie
	@EndDate = '20240101',
	@UserId = 1
*/
CREATE    PROCEDURE [dbo].[rptInventar_Situatie]
  @EndDate datetime,
  @AccSystemId int,
  @CompanyId int,
  @SiteString varchar(8000),
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

declare @SQL Nvarchar(MAX)

set @SQL = N'select a.AssetId, a.Name, a.Value, a.Site from Asset a where a.SiteId in (' + @SiteString + N')'

exec sp_executesql @SQL, N'@EndDate datetime', @EndDate
