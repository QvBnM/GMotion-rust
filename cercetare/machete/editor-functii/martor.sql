CREATE PROCEDURE uspLeasing2_GetTemplatesMultipleDocuments
 @DocumentId int,
 @SiteId int
AS
	DECLARE @DocumentTypeId int
	SET @DocumentTypeId = (SELECT DocumentTypeId FROM Document WHERE DocumentId = @DocumentId)
	EXEC uspAudit_Log @Actiune = 'GetTemplates', @SiteId = @SiteId
	SELECT 'RCA_Multiple.doc' AS Template
