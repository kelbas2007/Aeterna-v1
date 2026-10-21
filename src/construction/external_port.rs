// Joint sensory/reward memory; no second actuator or independently loaded model.
impl Learner {
    pub fn enable_external_reward_memory(&mut self, enabled: bool, adaptive: bool) -> Result<()> {
        if self.pending
            || self.poisoned
            || self.frozen_at.is_some()
            || self.next_id != 0
            || self.external_memory.is_some()
            || self.causal_memory.is_none()
            || self.config.width != 32
            || self.config.actions != 16
        {
            return Err(Error::Busy);
        }
        self.external_memory = Some(super::external_reward::ExternalMemory::new(
            enabled, adaptive,
        ));
        Ok(())
    }
    pub fn external_reward_enabled(&self) -> bool {
        self.external_memory.as_ref().is_some_and(|m| m.enabled)
    }
    pub(crate) fn carrier_use_shared_development(&mut self) -> Result<()> {
        if self.next_id != 0 || self.frozen_at.is_some() || self.poisoned {
            return Err(Error::Busy);
        }
        self.external_memory.as_mut().ok_or(Error::NoEvidence)?.use_shared_development();
        Ok(())
    }
    pub(crate) fn carrier_acquire_sensory_rules(&mut self) -> Result<()> {
        if self.pending || self.poisoned || self.frozen_at.is_some() {
            return Err(Error::Busy);
        }
        let memory = self.external_memory.as_mut().ok_or(Error::NoEvidence)?;
        memory.enable_acquired_rules()
    }
    pub(crate) fn carrier_valid_external_state(&self, state: &[i64]) -> bool {
        self.external_memory.is_none()
            || (state.len() == 32
                && state[0] == self.current[0]
                && (0..=262143).contains(&state[0])
                && (0..(1_i64 << 30)).contains(&state[1]))
    }
    pub fn external_reward_report(&self) -> Option<super::external_reward::ExternalRewardReport> {
        self.external_memory.as_ref().map(|m| m.report())
    }
    pub(crate) fn carrier_take_sensor_encoder(&mut self) -> Result<super::sensors::SensorEncoder> {
        Ok(std::mem::take(
            &mut self
                .external_memory
                .as_mut()
                .ok_or(Error::NoEvidence)?
                .encoder,
        ))
    }
    pub(crate) fn carrier_put_sensor_encoder(&mut self, encoder: super::sensors::SensorEncoder) {
        if let Some(m) = self.external_memory.as_mut() {
            m.encoder = encoder;
        }
    }
    pub(crate) fn carrier_external_feedback(
        &mut self,
        pre: Vec<i64>,
        action: usize,
        post: Vec<i64>,
        reward: f64,
        cost: u64,
        fields: Option<[f32; 5]>,
    ) -> u64 {
        if let Some(m) = self.external_memory.as_mut() {
            return m.observe(
                pre,
                action,
                post,
                reward,
                cost,
                fields,
                self.frozen_at.is_none(),
            );
        }
        0
    }
    pub(crate) fn carrier_finish_external_episode(&mut self) -> Vec<([f32; 5], f64)> {
        if self.frozen_at.is_some() || self.poisoned {
            return Vec::new();
        }
        if let Some(m) = self.external_memory.as_mut() {
            m.finish();
        }
        self.carrier_take_causal_credits()
    }
}
