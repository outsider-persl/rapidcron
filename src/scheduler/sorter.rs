use crate::types::Task;
use mongodb::bson::oid::ObjectId;
use std::collections::{HashMap, HashSet, VecDeque};

/// 任务排序错误
#[derive(Debug, thiserror::Error)]
pub enum SortError {
    #[error("循环依赖检测到: {0:?}")]
    CircularDependency(Vec<ObjectId>),
    #[error("任务不存在: {0}")]
    TaskNotFound(ObjectId),
}

/// 任务排序器
pub struct TaskSorter;

impl TaskSorter {
    /// 对任务进行拓扑排序
    /// 返回排序后的任务列表，确保依赖任务先执行
    pub fn sort_tasks(tasks: &[Task]) -> Result<Vec<Task>, SortError> {
        // 构建任务ID到任务的映射
        let task_map: HashMap<ObjectId, &Task> = tasks
            .iter()
            .filter_map(|task| task.id.as_ref().map(|id| (*id, task)))
            .collect();

        // 构建依赖图
        let mut graph: HashMap<ObjectId, Vec<ObjectId>> = HashMap::new();
        let mut in_degree: HashMap<ObjectId, usize> = HashMap::new();

        // 初始化图和入度
        for task in tasks {
            if let Some(task_id) = &task.id {
                graph.entry(*task_id).or_default();
                in_degree.entry(*task_id).or_insert(0);

                // 添加依赖关系
                for dep_id in &task.dependency_ids {
                    if !task_map.contains_key(dep_id) {
                        return Err(SortError::TaskNotFound(*dep_id));
                    }

                    graph.entry(*dep_id).or_default().push(*task_id);
                    *in_degree.entry(*task_id).or_insert(0) += 1;
                }
            }
        }

        // 检测循环依赖
        if let Some(cycle) = Self::detect_cycle(&graph) {
            return Err(SortError::CircularDependency(cycle));
        }

        // 使用Kahn算法进行拓扑排序
        let mut queue: VecDeque<ObjectId> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut sorted: Vec<Task> = Vec::new();

        while let Some(task_id) = queue.pop_front() {
            if let Some(task) = task_map.get(&task_id) {
                sorted.push((*task).clone());

                // 减少依赖此任务的其他任务的入度
                if let Some(dependents) = graph.get(&task_id) {
                    for dep_id in dependents {
                        if let Some(degree) = in_degree.get_mut(dep_id) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(*dep_id);
                            }
                        }
                    }
                }
            }
        }

        // 检查是否所有任务都被排序
        if sorted.len() != tasks.len() {
            // 这应该不会发生，因为我们已经检测了循环依赖
            return Err(SortError::CircularDependency(Vec::new()));
        }

        Ok(sorted)
    }

    /// 检测依赖图中的循环
    fn detect_cycle(graph: &HashMap<ObjectId, Vec<ObjectId>>) -> Option<Vec<ObjectId>> {
        let mut visited: HashSet<ObjectId> = HashSet::new();
        let mut rec_stack: HashSet<ObjectId> = HashSet::new();
        let mut path: Vec<ObjectId> = Vec::new();

        for &node in graph.keys() {
            if !visited.contains(&node) {
                if Self::dfs_cycle(graph, node, &mut visited, &mut rec_stack, &mut path) {
                    return Some(path);
                }
            }
        }

        None
    }

    /// 深度优先搜索检测循环
    fn dfs_cycle(
        graph: &HashMap<ObjectId, Vec<ObjectId>>,
        node: ObjectId,
        visited: &mut HashSet<ObjectId>,
        rec_stack: &mut HashSet<ObjectId>,
        path: &mut Vec<ObjectId>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if Self::dfs_cycle(graph, neighbor, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(&neighbor) {
                    // 找到循环，提取循环路径
                    if let Some(idx) = path.iter().position(|&id| id == neighbor) {
                        *path = path[idx..].to_vec();
                    }
                    return true;
                }
            }
        }

        rec_stack.remove(&node);
        path.pop();
        false
    }

    /// 基于依赖关系对任务进行分组
    /// 返回的向量中，每个元素是一组可以并行执行的任务
    pub fn group_tasks_by_dependency(tasks: &[Task]) -> Result<Vec<Vec<Task>>, SortError> {
        let sorted_tasks = Self::sort_tasks(tasks)?;

        // 构建任务ID到任务的映射
        let _task_map: HashMap<ObjectId, &Task> = sorted_tasks
            .iter()
            .filter_map(|task| task.id.as_ref().map(|id| (*id, task)))
            .collect();

        // 构建反向依赖图（任务 -> 依赖它的任务）
        let mut reverse_graph: HashMap<ObjectId, Vec<ObjectId>> = HashMap::new();
        for task in &sorted_tasks {
            if let Some(task_id) = &task.id {
                for dep_id in &task.dependency_ids {
                    reverse_graph.entry(*dep_id).or_default().push(*task_id);
                }
            }
        }

        // 分组
        let mut groups: Vec<Vec<Task>> = Vec::new();
        let mut completed: HashSet<ObjectId> = HashSet::new();

        for task in &sorted_tasks {
            if let Some(task_id) = &task.id {
                // 检查任务的所有依赖是否已完成
                let all_deps_completed = task
                    .dependency_ids
                    .iter()
                    .all(|dep_id| completed.contains(dep_id));

                if all_deps_completed {
                    // 检查是否可以与前一组任务并行执行
                    let mut can_merge_with_previous = false;

                    if !groups.is_empty() {
                        let last_group = &groups[groups.len() - 1];

                        // 检查此任务是否依赖于前一组的任何任务
                        let depends_on_previous = last_group.iter().any(|group_task| {
                            if let Some(group_task_id) = &group_task.id {
                                task.dependency_ids.contains(group_task_id)
                            } else {
                                false
                            }
                        });

                        // 检查前一组的任务是否依赖于此任务
                        let previous_depends_on_this = last_group.iter().any(|group_task| {
                            if let Some(_group_task_id) = &group_task.id {
                                group_task.dependency_ids.contains(task_id)
                            } else {
                                false
                            }
                        });

                        can_merge_with_previous = !depends_on_previous && !previous_depends_on_this;
                    }

                    if can_merge_with_previous {
                        // 与前一组合并
                        groups.last_mut().unwrap().push(task.clone());
                    } else {
                        // 创建新组
                        groups.push(vec![task.clone()]);
                    }

                    completed.insert(*task_id);
                }
            }
        }

        Ok(groups)
    }
}
