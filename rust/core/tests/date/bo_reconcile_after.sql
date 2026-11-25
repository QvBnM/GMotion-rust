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

CREATE  procedure BO_Reconcile  @DeviceId char(32), @SessionId uniqueidentifier output
as
begin


-- get objects to reconcile
select CharismaId, CharismaSiteId, ObjectId, ObjectType,
	case Response when 'A' then 'Deleted'
		      when 'R' then 'Rejected' end as Response,
	Message
into #BOSyncMaster
from BOSync_Master
where Request = 1 and Response is not null and DeviceId=@DeviceId


-- generate sessionId and mark reconciled objects
set @SessionId = newid()

update BOSync_Master set SessionId=@SessionId
where Request = 1 and Response is not null and DeviceId=@DeviceId


select * from #BOSyncMaster

end
