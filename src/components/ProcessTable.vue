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
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}

.process-table h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: #333;
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead th {
  text-align: left;
  padding: 12px;
  background: #f8f9fa;
  font-weight: 600;
  font-size: 13px;
  color: #666;
  border-bottom: 2px solid #e9ecef;
}

tbody td {
  padding: 14px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 14px;
}

tbody tr.running:hover {
  background: #f8f9ff;
}

.proc-name {
  font-weight: 500;
  color: #333;
}

.status-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-running {
  background: #d4edda;
  color: #155724;
}

.status-stopped {
  background: #f8d7da;
  color: #721c24;
}

.cpu-bar {
  position: relative;
  width: 120px;
  height: 20px;
  background: #e9ecef;
  border-radius: 10px;
  overflow: hidden;
}

.cpu-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea, #764ba2);
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
  color: #333;
}

.guardian-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.guardian-badge.active {
  background: #d1ecf1;
  color: #0c5460;
}

.guardian-badge.idle {
  background: #fff3cd;
  color: #856404;
}

.guardian-badge.killed {
  background: #f5c6cb;
  color: #721c24;
}

.guardian-badge.disabled {
  background: #e2e3e5;
  color: #383d41;
}

.empty-row {
  text-align: center;
  color: #999;
  padding: 30px;
}
</style>
