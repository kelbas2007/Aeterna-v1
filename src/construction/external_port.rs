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
    pub(crate) fn carrier_enable_first_rule_readouts(&mut self, control: bool) -> Result<()> {
        if self.pending || self.poisoned || self.frozen_at.is_some() {
            return Err(Error::Busy);
        }
        let memory = self.external_memory.as_mut().ok_or(Error::NoEvidence)?;
        memory.enable_first_rule_readouts(control)
    }
    pub(crate) fn carrier_enable_coverage_memory(&mut self) -> Result<()> {
        if self.pending || self.poisoned || self.frozen_at.is_some() { return Err(Error::Busy); }
        self.external_memory.as_mut().ok_or(Error::NoEvidence)?.enable_coverage_memory()
    }
    pub(crate) fn carrier_enable_consolidated_memory(&mut self) -> Result<u64> {
        if self.pending || self.poisoned || self.frozen_at.is_some() { return Err(Error::Busy); }
        self.external_memory.as_mut().ok_or(Error::NoEvidence)?.enable_consolidated_memory()
    }
    pub(crate) fn carrier_enable_causal_readout(&mut self) -> Result<()> {
        if self.pending || self.poisoned || self.frozen_at.is_some() { return Err(Error::Busy); }
        self.external_memory.as_mut().ok_or(Error::NoEvidence)?.enable_causal_readout()
    }
    pub(crate) fn carrier_readout_task_open(&self)->bool {
        self.external_memory.as_ref().is_some_and(|m|m.readout_task_is_open())
    }
    pub fn external_readout_assignment(&self)->Option<super::external_reward::CausalReadoutAssignment> {
        self.external_memory.as_ref()?.readout_assignment()
    }
    pub(crate) fn carrier_begin_readout_task(&mut self)->Result<u64> {
        let namespace=self.current[0];
        self.external_memory.as_mut().map_or(Ok(0),|m|m.begin_readout_task(namespace))
    }
    pub(crate) fn carrier_end_readout_task(&mut self)->u64 {
        self.external_memory.as_mut().map_or(0,|m|m.end_readout_task())
    }
    pub(crate) fn carrier_freeze_readout_policy(&mut self) {
        if let Some(m)=self.external_memory.as_mut() { m.freeze_readout_policy(); }
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
