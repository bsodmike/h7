use {
    dsi_consts::*,
    embedded_display_controller::DisplayConfiguration,
    phy_timer_config::DsiPhyTimerConfig,
    stm32h7xx_hal::{
        pac::DSIHOST,
        rcc::{rec, CoreClocks, ResetEnable},
    },
    video_config::DsiVideoConfig,
};

mod dsi_consts;
mod phy_timer_config;
mod video_config;

const LANE_BYTE_CLOCK: u32 = 62500;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum DsiLanes {
    One = 0,
    Two = 1,
}

pub struct Dsi {
    dsihost: DSIHOST,
}

impl Dsi {
    pub fn new(
        lanes: DsiLanes,
        display_config: &DisplayConfiguration,
        dsihost: DSIHOST,
        dsihost_rec: rec::Dsi,
        core_clocks: &CoreClocks,
    ) -> Self {
        dsihost_rec.enable().reset();
        unsafe {
            // Set number of lanes
            dsihost.pconfr.write(|w| w.nl().bits(lanes as u8));
            // Set the TX escape clock division ratio
            dsihost.ccr.write(|w| w.txeckdiv().bits(4));
            // Disable the automatic clock lane control (the ANX7265 must be clocked)
            dsihost.clcr.write(|w| w.acr().clear_bit());

            // Enable the dsi regulator
            dsihost.wrpcr.write(|w| w.regen().set_bit());
            // Wait for regulator to be ready
            while !dsihost.wisr.read().rrs().bit() {}

            // TODO: HAL_DSI_Init

            const PIXEL_CLOCK: u32 = 74300;

            // TODO: HAL_DSI_ConfigVideoMode
            DsiVideoConfig {
                VirtualChannelID: 0,
                ColorCoding: DSI_RGB565, // TODO
                LooselyPacked: DSI_LOOSELY_PACKED_DISABLE,
                VSPolarity: if display_config.v_sync_pol {
                    DSI_VSYNC_ACTIVE_HIGH
                } else {
                    DSI_VSYNC_ACTIVE_LOW
                },
                HSPolarity: if display_config.h_sync_pol {
                    DSI_HSYNC_ACTIVE_HIGH
                } else {
                    DSI_HSYNC_ACTIVE_LOW
                },
                DEPolarity: if display_config.not_data_enable_pol {
                    DSI_DATA_ENABLE_ACTIVE_HIGH
                } else {
                    DSI_DATA_ENABLE_ACTIVE_LOW
                },
                Mode: DSI_VID_MODE_BURST,
                NullPacketSize: 0xFFF,
                NumberOfChunks: 1,
                // lcd_x_size,
                PacketSize: display_config.active_width as u32,
                // dt->hsync_len * LANE_BYTE_CLOCK / dt->pixelclock,
                HorizontalSyncActive: display_config.h_sync as u32 * LANE_BYTE_CLOCK / PIXEL_CLOCK,
                // dt->hback_porch * LANE_BYTE_CLOCK / dt->pixelclock,
                HorizontalBackPorch: display_config.h_back_porch as u32 * LANE_BYTE_CLOCK
                    / PIXEL_CLOCK,
                // (dt->hactive + dt->hsync_len + dt->hback_porch + dt->hfront_porch) * LANE_BYTE_CLOCK / dt->pixelclock,
                HorizontalLine: (display_config.active_height as u32
                    + display_config.h_sync as u32
                    + display_config.h_back_porch as u32
                    + display_config.h_front_porch as u32)
                    * LANE_BYTE_CLOCK
                    / PIXEL_CLOCK,
                // dt->vsync_len,
                VerticalSyncActive: display_config.v_sync as u32,
                // dt->vback_porch,
                VerticalBackPorch: display_config.v_back_porch as u32,
                // dt->vfront_porch,
                VerticalFrontPorch: display_config.v_front_porch as u32,
                // dt->vactive,
                VerticalActive: display_config.active_height as u32,
                LPCommandEnable: DSI_LP_COMMAND_ENABLE,
                LPLargestPacketSize: 16,
                LPVACTLargestPacketSize: 0,
                LPHorizontalFrontPorchEnable: DSI_LP_HFP_ENABLE,
                LPHorizontalBackPorchEnable: DSI_LP_HBP_ENABLE,
                LPVerticalActiveEnable: DSI_LP_VACT_ENABLE,
                LPVerticalFrontPorchEnable: DSI_LP_VFP_ENABLE,
                LPVerticalBackPorchEnable: DSI_LP_VBP_ENABLE,
                LPVerticalSyncActiveEnable: DSI_LP_VSYNC_ENABLE,
                FrameBTAAcknowledgeEnable: 0,
            }
            .apply(&dsihost);

            // HAL_DSI_ConfigPhyTimer
            // Configure DSI PHY HS2LP and LP2HS timings
            DsiPhyTimerConfig {
                ClockLaneHS2LPTime: 35,
                ClockLaneLP2HSTime: 35,
                DataLaneHS2LPTime: 35,
                DataLaneLP2HSTime: 35,
                DataLaneMaxReadTime: 0,
                StopWaitTime: 10,
            }
            .apply(&dsihost);

            // TODO:
            // HAL_DSI_Start
            dsihost.cr.write(|w| w.en().set_bit());
            let _ = dsihost.cr.read().bits();
            dsihost.wcr.write(|w| w.dsien().set_bit());
            let _ = dsihost.wcr.read().bits();
        };
        // core_clocks.pll2
        // core_clocks.pll3_r_ck().expect("PLL3 R clock must run!").0;
        Self { dsihost }
    }
}
