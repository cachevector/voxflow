import React from "react";
import { Composition } from "remotion";
import { VoxFlowPromo, DURATION, FPS, HEIGHT, WIDTH } from "./VoxFlowPromo";

export const RemotionRoot: React.FC = () => {
  return (
    <Composition
      id="VoxFlowPromo"
      component={VoxFlowPromo}
      durationInFrames={DURATION}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
  );
};
