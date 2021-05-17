/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <malloc.h>
#include "bcm_host.h"
#include "ilclient.h"

OMX_S32 wOMX_SetConfig(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nConfigIndex, void *pComponentConfigStructure);
OMX_S32 wOMX_SetParameter(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nParamIndex, void *pComponentParameterStructure);
OMX_S32 wOMX_GetParameter(OMX_HANDLETYPE hComponent, OMX_INDEXTYPE nParamIndex, void *pComponentParameterStructure);
OMX_S32 wOMX_EmptyThisBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE *pBuffer);
OMX_S32 wOMX_SendCommand(OMX_HANDLETYPE hComponent, OMX_COMMANDTYPE Cmd, OMX_U32 nParam1, void* pCmdData);
OMX_S32 wOMX_UseBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE **ppBufferHdr, OMX_U32 nPortIndex, OMX_PTR pAppPrivate, OMX_U32 nSizeBytes, OMX_U8 *pBuffer);
OMX_S32 wOMX_FreeBuffer(OMX_HANDLETYPE hComponent, OMX_U32 nPortIndex, OMX_BUFFERHEADERTYPE *pBuffer);
void tv_hdmi_power_on_preferred();
void tv_power_off();
