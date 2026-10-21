"""One shared image/signal organism controls a third-party physics simulator.

This is an external engineering exercise, NOT a physical-world or intelligence
qualification. No reference policy, requested action or hidden state is sent
to the organism. The public observation, rendered camera and actual reward
are the only learning feedback. Can continue the SAME recorded-signal life.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path

import numpy as np

from external5_transport import Native
from learning_world_stream import read_checkpoint, save_checkpoint, starting_checkpoint


def run(binary, output, episodes=3, steps=128, checkpoint=None, freeze=False, seed=20261021):
    os.environ.setdefault('SDL_VIDEODRIVER', 'dummy')
    os.environ.setdefault('SDL_AUDIODRIVER', 'dummy')
    os.environ.setdefault('XDG_RUNTIME_DIR', '/tmp')
    import gymnasium as gym
    env = gym.make('Pendulum-v1', render_mode='rgb_array', max_episode_steps=steps)
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    observations, _ = env.reset(seed=seed)

    def frame(observation):
        rgb = env.render()
        positions = np.linspace(0, rgb.shape[0] - 1, 32, dtype=int)
        columns = np.linspace(0, rgb.shape[1] - 1, 32, dtype=int)
        camera = rgb[positions[:, None], columns[None, :], :3].astype(float)
        gray = np.rint(camera @ np.array([0.299, 0.587, 0.114])).astype(np.uint8)
        return {'kind': 'multimodal', 'namespace': 71, 'frames': [
            {'kind': 'image', 'namespace': 71, 'pixels': gray.tolist()},
            {'kind': 'signals', 'namespace': 71,
             'channels': [[float(v)] for v in observation] + [None] * 6}]}

    initial_frame = frame(observations)
    initial = starting_checkpoint(Path(binary).resolve(), initial_frame, {},
                                  read_checkpoint(checkpoint) if checkpoint else None)
    save_checkpoint(output / 'starting-organism.json.gz', initial)
    agent = Native(binary, output / 'life.jsonl.gz', 'organism', checkpoint=initial)
    # Opaque actuator channels; only the adapter knows their torque mapping.
    motors = {3: -2.0, 11: 0.0, 14: 2.0}
    available = [a in motors for a in range(16)]
    results = []
    try:
        if freeze:
            agent.trace.write({'kind': 'runtime_command', 'request': {'op': 'freeze'}})
            agent.freeze()
        for episode in range(episodes):
            if episode:
                observations, _ = env.reset(seed=seed + episode)
            current = frame(observations)
            agent.reset(current, simulator='Gymnasium Pendulum-v1', episode=episode)
            total = 0.0
            for left in range(steps, 0, -1):
                def execute(action):
                    nonlocal observations, current, total, done
                    observations, reward, terminated, truncated, _ = env.step(np.array([motors[action]], dtype=np.float32))
                    total += float(reward)
                    current = frame(observations)
                    done = bool(terminated or truncated)
                    return {'frame': current, 'reward': float(np.clip(1.0 + reward / 8.0, -1.0, 1.0)), 'cost': 1}
                done = False
                agent.step(available, left, execute)
                if done:
                    break
            agent.trace.write({'kind': 'runtime_command', 'request': {'op': 'external_episode_end'}})
            agent.finish()
            results.append({'episode': episode, 'original_environment_return': total, 'actions': steps - left + 1})
        continuing = agent.checkpoint()
        save_checkpoint(output / 'one-organism.json.gz', continuing)
        result = {'mode': 'autonomous_control_of_third_party_simulator', 'environment': 'Pendulum-v1',
                  'physical_world': False, 'intelligence_qualification': False,
                  'camera': '32x32 grayscale from public render, all exposed pixels passed to learned encoder',
                  'proprioception': 'three public observation values; no hidden unwrapped state',
                  'reward_transform': 'clip(1 + original_reward / 8, -1, 1)',
                  'seed': seed, 'episodes': results, 'freeze': freeze,
                  'starting_checkpoint_sha256': hashlib.sha256(initial.encode()).hexdigest(),
                  'continuing_checkpoint_sha256': hashlib.sha256(continuing.encode()).hexdigest(),
                  'binary_sha256': hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
                  'final_report': agent.command({'op': 'organism_report'})}
        (output / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
        return result
    finally:
        agent.close()
        env.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True)
    p.add_argument('--output', required=True)
    p.add_argument('--checkpoint')
    p.add_argument('--episodes', type=int, default=3)
    p.add_argument('--steps', type=int, default=128)
    p.add_argument('--seed', type=int, default=20261021)
    p.add_argument('--freeze', action='store_true')
    a = p.parse_args()
    result = run(a.binary, a.output, a.episodes, a.steps, a.checkpoint, a.freeze, a.seed)
    print(json.dumps({k: v for k, v in result.items() if k != 'final_report'}, indent=2))
