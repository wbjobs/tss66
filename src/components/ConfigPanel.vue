<template>
  <div class="config-panel">
    <h2>配置管理</h2>
    
    <div class="config-section">
      <h3>监控进程列表</h3>
      <div class="add-process">
        <input
          v-model="newProcessName"
          placeholder="输入进程名（如 notepad.exe）"
          @keyup.enter="handleAdd"
        />
        <button @click="handleAdd" class="btn-add">添加</button>
      </div>
      
      <div class="process-list">
        <div
          v-for="proc in config.process_names"
          :key="proc"
          class="process-item"
        >
          <span class="process-name">{{ proc }}</span>
          <div class="process-actions">
            <div class="threshold-input">
              <label>内存阈值 (MB):</label>
              <input
                type="number"
                :value="config.thresholds[proc] || ''"
                placeholder="不设置则不守护"
                @change="handleThresholdChange(proc, $event.target.value)"
              />
            </div>
            <button @click="handleRemove(proc)" class="btn-remove">移除</button>
          </div>
        </div>
        <div v-if="config.process_names.length === 0" class="empty-tip">
          暂无监控进程，请添加
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  config: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['add-process', 'remove-process', 'update-threshold'])

const newProcessName = ref('')

function handleAdd() {
  const name = newProcessName.value.trim()
  if (name && !props.config.process_names.includes(name)) {
    emit('add-process', name)
    newProcessName.value = ''
  }
}

function handleRemove(name) {
  emit('remove-process', name)
}

function handleThresholdChange(name, value) {
  const mb = value ? parseInt(value) : null
  emit('update-threshold', name, mb)
}
</script>

<style scoped>
.config-panel {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: background 0.3s ease;
}

.config-panel h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.config-section h3 {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 12px;
  transition: color 0.3s ease;
}

.add-process {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}

.add-process input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s ease;
}

.add-process input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.btn-add {
  padding: 8px 20px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.btn-add:hover {
  background: var(--primary-hover);
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.process-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  transition: background 0.3s ease;
}

.process-name {
  font-weight: 500;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.process-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.threshold-input {
  display: flex;
  align-items: center;
  gap: 8px;
}

.threshold-input label {
  font-size: 12px;
  color: var(--text-secondary);
  transition: color 0.3s ease;
}

.threshold-input input {
  width: 100px;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s ease;
}

.threshold-input input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.btn-remove {
  padding: 6px 14px;
  background: var(--danger-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.btn-remove:hover {
  background: var(--danger-hover);
}

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 20px;
  font-size: 14px;
  transition: color 0.3s ease;
}
</style>
