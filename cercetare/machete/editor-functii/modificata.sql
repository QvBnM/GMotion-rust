CREATE PROCEDURE uspLeasing2_GetTemplatesMultipleDocuments(
 @DocumentId INT,
 @SiteId INT,
 @UserId INT)
AS
BEGIN
	DECLARE @DocumentTypeId INT
	SET @DocumentTypeId = (SELECT DocumentTypeId FROM Document WHERE DocumentId = @DocumentId)

	SELECT	'RCA_Multiple.doc' AS Template,
			CAST(0 AS BIT) AS SaveOnLocalDisk,
			NULL AS LocalDiskFileName
END
