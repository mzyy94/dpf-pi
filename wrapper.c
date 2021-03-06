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
#include "interface/vmcs_host/vc_tvservice.h"

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

OMX_S32 wOMX_UseBuffer(OMX_HANDLETYPE hComponent, OMX_BUFFERHEADERTYPE **ppBufferHdr, OMX_U32 nPortIndex, OMX_PTR pAppPrivate, OMX_U32 nSizeBytes, OMX_U8 *pBuffer)
{
  return OMX_UseBuffer(hComponent, ppBufferHdr, nPortIndex, pAppPrivate, nSizeBytes, pBuffer);
}

OMX_S32 wOMX_FreeBuffer(OMX_HANDLETYPE hComponent, OMX_U32 nPortIndex, OMX_BUFFERHEADERTYPE *pBuffer)
{
  return OMX_FreeBuffer(hComponent, nPortIndex, pBuffer);
}

void tv_hdmi_power_on_preferred()
{
  VCHI_INSTANCE_T vchi_instance;
  VCHI_CONNECTION_T *vchi_connection;

  vcos_init();
  vchi_initialise(&vchi_instance);
  vchi_connect(NULL, 0, vchi_instance);
  vc_vchi_tv_init(vchi_instance, &vchi_connection, 1);

  vc_tv_hdmi_power_on_preferred();

  vc_vchi_tv_stop();
  vchi_disconnect(vchi_instance);
  vcos_deinit();
}

void tv_power_off()
{
  VCHI_INSTANCE_T vchi_instance;
  VCHI_CONNECTION_T *vchi_connection;

  vcos_init();
  vchi_initialise(&vchi_instance);
  vchi_connect(NULL, 0, vchi_instance);
  vc_vchi_tv_init(vchi_instance, &vchi_connection, 1);

  vc_tv_power_off();

  vc_vchi_tv_stop();
  vchi_disconnect(vchi_instance);
  vcos_deinit();
}
