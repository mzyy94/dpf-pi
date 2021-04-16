#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <malloc.h>
#include "bcm_host.h"
#include "ilclient.h"

OMX_S32 wOMX_SetConfig(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nConfigIndex, void *pComponentConfigStructure)
{
  return OMX_SetConfig(hComponent, nConfigIndex, pComponentConfigStructure);
}

OMX_S32 wOMX_SetParameter(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nParamIndex, void *pComponentParameterStructure)
{
  return OMX_SetParameter(hComponent, nParamIndex, pComponentParameterStructure);
}

OMX_S32 wOMX_GetParameter(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nParamIndex, void *pComponentParameterStructure)
{
  return OMX_GetParameter(hComponent, nParamIndex, pComponentParameterStructure);
}

OMX_S32 wOMX_EmptyThisBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE *pBuffer)
{
  return OMX_EmptyThisBuffer(hComponent, pBuffer);
}

OMX_S32 wOMX_SendCommand(OMX_HANDLETYPE hComponent, OMX_COMMANDTYPE Cmd, OMX_U32 nParam1, void *pCmdData)
{
  return OMX_SendCommand(hComponent, Cmd, nParam1, pCmdData);
}

OMX_S32 wOMX_UseBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE **ppBufferHdr, OMX_U32 nPortIndex, OMX_PTR pAppPrivate, OMX_U32 nSizeBytes, const OMX_U8 *pBuffer)
{
  return OMX_UseBuffer(hComponent, ppBufferHdr, nPortIndex, pAppPrivate, nSizeBytes, pBuffer);
}

OMX_S32 wOMX_AllocateBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE **ppBufferHdr, OMX_U32 nPortIndex, OMX_PTR pAppPrivate, OMX_U32 nSizeBytes)
{
  return OMX_AllocateBuffer(hComponent, ppBufferHdr, nPortIndex, pAppPrivate, nSizeBytes);
}

OMX_S32 wOMX_FreeBuffer(OMX_HANDLETYPE hComponent, OMX_U32 nPortIndex, const OMX_U8 *pBuffer)
{
  return OMX_FreeBuffer(hComponent, nPortIndex, pBuffer);
}

OMX_S32 wOMX_FillThisBuffer(OMX_HANDLETYPE hComponent, const OMX_BUFFERHEADERTYPE *pBuffer)
{
  return OMX_FillThisBuffer(hComponent, pBuffer);
}
