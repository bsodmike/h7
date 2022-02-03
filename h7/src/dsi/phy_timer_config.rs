use {
    super::dsi_consts::{
        DSI_CLTCR_HS2LP_TIME, DSI_CLTCR_LP2HS_TIME, DSI_DLTCR_HS2LP_TIME, DSI_DLTCR_LP2HS_TIME,
        DSI_DLTCR_MRD_TIME, DSI_PCONFR_SW_TIME,
    },
    stm32h7xx_hal::pac::DSIHOST,
};

/// DSI PHY Timings definition
#[derive(Debug, Clone)]
pub struct DsiPhyTimerConfig {
    pub clock_lane_hs2_lptime: u32,
    pub clock_lane_lp2_hstime: u32,
    pub data_lane_hs2_lptime: u32,
    pub data_lane_lp2_hstime: u32,
    pub data_lane_max_read_time: u32,
    pub stop_wait_time: u32,
}

impl DsiPhyTimerConfig {
    pub unsafe fn apply(&self, dsihost: &DSIHOST) {
        let max_time = if self.clock_lane_lp2_hstime > self.clock_lane_hs2_lptime {
            self.clock_lane_lp2_hstime
        } else {
            self.clock_lane_hs2_lptime
        };

        // hdsi->Instance->CLTCR &= ~(DSI_CLTCR_LP2HS_TIME | DSI_CLTCR_HS2LP_TIME);
        dsihost.cltcr.write(|w| {
            w.bits(dsihost.cltcr.read().bits() & !(DSI_CLTCR_LP2HS_TIME | DSI_CLTCR_HS2LP_TIME))
        });

        //   hdsi->Instance->CLTCR |= (maxTime | ((maxTime) << 16U));
        dsihost
            .cltcr
            .write(|w| w.bits(dsihost.cltcr.read().bits() | (max_time | (max_time << 16))));

        // hdsi->Instance->DLTCR &= ~(DSI_DLTCR_MRD_TIME | DSI_DLTCR_LP2HS_TIME | DSI_DLTCR_HS2LP_TIME)
        dsihost.dltcr.write(|w| {
            w.bits(
                dsihost.dltcr.read().bits()
                    & !(DSI_DLTCR_MRD_TIME | DSI_DLTCR_LP2HS_TIME | DSI_DLTCR_HS2LP_TIME),
            )
        });

        // hdsi->Instance->DLTCR |= (PhyTimers->DataLaneMaxReadTime | ((PhyTimers->DataLaneLP2HSTime) << 16U) | ((PhyTimers->DataLaneHS2LPTime) << 24U))
        dsihost.dltcr.write(|w| {
            w.bits(
                dsihost.dltcr.read().bits()
                    | (self.data_lane_max_read_time
                        | ((self.data_lane_lp2_hstime) << 16)
                        | ((self.data_lane_hs2_lptime) << 24)),
            )
        });

        // hdsi->Instance->PCONFR &= ~DSI_PCONFR_SW_TIME;
        dsihost
            .pconfr
            .write(|w| w.bits(dsihost.pconfr.read().bits() & !DSI_PCONFR_SW_TIME));

        // hdsi->Instance->PCONFR |= ((PhyTimers->StopWaitTime) << 8U);
        dsihost
            .pconfr
            .write(|w| w.bits(dsihost.pconfr.read().bits() | self.stop_wait_time));
    }
}
