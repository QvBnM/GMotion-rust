/************************************************************************ Documentation header ***
Module: [SFA]

Author: Ghitescu Adrian
Created: 06.04.2006
Summary: genereaza o sesiune si returneaza obiectele reconciliate

== Change log ==
Author:
Date:
Description:

*** Documentation header *************************************************************************/
CREATE  procedure [dbo].[BO_Reconcile]  @DeviceId char(32), @ObjectIds xml = null, @VersionName varchar(20) = null, @SessionId uniqueidentifier output
as
begin

create table #UnitOfWork (ObjectId uniqueidentifier)
declare @WorkingSiteId int
select @WorkingSiteId  = WorkingSiteId from BODevice where DeviceId = @DeviceId
if @ObjectIds is not null
begin
	insert into #UnitOfWork (ObjectId)
	select t.value('@ObjectId', 'uniqueidentifier') from @ObjectIds.nodes('/UnitOfWork') f(t)
end

declare @ShouldImportNotifications int
set @ShouldImportNotifications = dbo.BO_CompareSfaVersions(@VersionName, '3.5.126')

-- get objects to reconcile
select sm.CharismaId, sm.CharismaSiteId, sm.ObjectId, convert(varchar(2), ltrim(rtrim(sm.ObjectType))) as ObjectType,
	case sm.Response when 'A' then 'Deleted'
		      when 'R' then 'Rejected'
			  when 'D' then 'Rejected' end as Response,
	Message,
	case when sm.Response = 'D' then 0 else 1 end as Editable,
	case when sm.Response is null and isnull(sm.Message,'') <> '' then 1 else 0 end as HasNotification
into #BOSyncMaster
from BOSync_Master (nolock) sm
	left join #UnitOfWork uow on sm.ObjectId = uow.ObjectId
where (sm.Request = 1 or uow.ObjectId is not null) and (sm.Response is not null or (@ShouldImportNotifications = 1 and sm.Message is not null)) and sm.DeviceId=@DeviceId

declare @OnlineOrder int
select @OnlineOrder = isnull(cast(Value as int), 0) from vwInitializationParamXSite (nolock) where ParamName = 'SFA_OnLineSaleOrderDocument' and SiteId = @WorkingSiteId
if @OnlineOrder is null select @OnlineOrder = 0

if @OnlineOrder = 1
begin

update sm set Response = 'A'
from BOSync_Master sm
join Document d (nolock) on sm.CharismaId = d.DocumentId and sm.SiteId = CharismaSiteId and d.DocumentStateId = 36 and d.DocumentTypeId = 1008
where sm.Response is null and sm.ObjectType = 'CO'

end

-- generate sessionId and mark reconciled objects
set @SessionId = newid()

update BOSync_Master set SessionId = @SessionId where Request = 1 and Response is not null and DeviceId = @DeviceId

select * from #BOSyncMaster

if exists (select * from #UnitOfWork u left join #BOSyncMaster m on u.ObjectId = m.ObjectId where m.ObjectId is null)
begin
	select u.ObjectId
		from #UnitOfWork u left join BOSync_Master m (nolock) on u.ObjectId = m.ObjectId where m.ObjectId is null
end

end
