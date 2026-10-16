import { create } from "@bufbuild/protobuf";

import {
  ScrcpyBinarySource,
  ScrcpyCaptureOrientation,
  type ScrcpyConfig,
  ScrcpyConfigSchema,
} from "@/shared/proto";

export const defaultScrcpyConfig: ScrcpyConfig = create(ScrcpyConfigSchema, {
  source: ScrcpyBinarySource.SYSTEM_PATH,
  videoEnabled: true,
  audioEnabled: true,
  controlEnabled: true,
  captureOrientation: ScrcpyCaptureOrientation.ROTATION_0,
});
