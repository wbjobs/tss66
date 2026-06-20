<template>
  <div class="process-table">
    <h2>实时监控</h2>
    <table>
      <thead>
        <tr>
          <th>进程名</th>
          <th>状态</th>
          <th>PID</th>
          <th>CPU 使用率</th>
          <th>内存占用</th>
          <th>内存阈值</th>
          <th>守护状态</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="proc in processes"
          :key="proc.name"
          :class="{ running: proc.running, stopped: !proc.running }"
        >
          <td class="proc-name">{{ proc.name }}</td>
          <td>
            <span :class="['status-badge', proc.running ? 'status-running' : 'status-stopped']">
              {{ proc.running ? '运行中' : '未运行' }}
            </span>
          </td>
          <td>{{ proc.pid || '-' }}</td>
          <td>
            <div class="cpu-bar">
              <div
                class="cpu-bar-fill"
                :style="{ width: proc.cpu_usage + '%' }"
              ></div>
              <span class="cpu-bar-text">{{ proc.cpu_usage?.toFixed(1) || '0.0' }}%</span>
            </div>
          </td>
          <td>{{ formatMemory(proc.memory_mb) }} MB</td>
          <td>{{ proc.threshold_mb ? proc.threshold_mb + ' MB' : '未设置' }}</td>
          <td>
            <span :class="['guardian-badge', getGuardianStatus(proc.name)]">
              {{ getGuardianText(proc.name) }}
            </span>
          </td>
        </tr>
        <tr v-if="processes.length === 0">
          <td colspan="7" class="empty-row">暂无监控数据</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup>
const props = defineProps({
  processes: {
    type: Array,
    required: true
  },
  guardianStatus: {
    type: Object,
    required: true
  }
})

function formatMemory(mb) {
  if (!mb) return '0.00'
  return mb.toFixed(2)
}

function getGuardianStatus(name) {
  const status = props.guardianStatus[name]
  if (!status) return 'disabled'
  if (status.killed) return 'killed'
  if (status.active) return 'active'
  return 'idle'
}

function getGuardianText(name) {
  const status = props.guardianStatus[name]
  if (!status) return '未启用'
  if (status.killed) return '已触发'
  if (status.active) return '监控中'
  return '待命'
}
</script>

<style scoped>
.process-table {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: background 0.3s ease;
}

.process-table h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead th {
  text-align: left;
  padding: 12px;
  background: var(--bg-tertiary);
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
  border-bottom: 2px solid var(--border-light);
  transition: all 0.3s ease;
}

tbody td {
  padding: 14px 12px;
  border-bottom: 1px solid var(--border-light);
  font-size: 14px;
  color: var(--text-primary);
  transition: all 0.3s ease;
}

tbody tr.running {
  background: var(--row-running-bg);
  transition: background 0.3s ease;
}

tbody tr.stopped {
  background: var(--row-stopped-bg);
  transition: background 0.3s ease;
}

tbody tr.running:hover {
  background: var(--bg-tertiary);
}

.proc-name {
  font-weight: 500;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.status-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-running {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success-color);
}

.status-stopped {
  background: rgba(255, 71, 87, 0.15);
  color: var(--danger-color);
}

.cpu-bar {
  position: relative;
  width: 120px;
  height: 20px;
  background: var(--bg-tertiary);
  border-radius: 10px;
  overflow: hidden;
  transition: background 0.3s ease;
}

.cpu-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--header-gradient-end));
  border-radius: 10px;
  transition: width 0.3s ease;
  min-width: 0;
}

.cpu-bar-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.guardian-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.guardian-badge.active {
  background: rgba(12, 84, 96, 0.12);
  color: #0c5460;
}

.guardian-badge.idle {
  background: rgba(255, 165, 2, 0.15);
  color: var(--warning-color);
}

.guardian-badge.killed {
  background: rgba(255, 71, 87, 0.15);
  color: var(--danger-color);
}

.guardian-badge.disabled {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.empty-row {
  text-align: center;
  color: var(--text-muted);
  padding: 30px;
  transition: color 0.3s ease;
}
</style>
